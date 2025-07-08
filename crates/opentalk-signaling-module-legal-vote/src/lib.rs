// SPDX-FileCopyrightText: OpenTalk GmbH <mail@opentalk.eu>
//
// SPDX-License-Identifier: EUPL-1.2

//! # Legal Vote Module
//!
//! ## Functionality
//!
//! Offers full legal vote features including live voting with high safety guards (atomic changes, audit log).
//! Stores the result for further archival storage in postgres.

use std::{
    collections::{BTreeMap, BTreeSet, HashMap},
    path::Path,
    sync::Arc,
};

use bytes::Bytes;
use chrono::{DateTime, Utc};
use chrono_tz::Tz;
use either::Either;
use error::LegalVoteError;
use futures::{FutureExt, stream::once};
use kustos::{Authz, Resource, prelude::AccessMethod};
use opentalk_database::Db;
use opentalk_db_storage::{
    module_resources::{Filter, ModuleResource, NewModuleResource},
    rooms::Room,
    users::User,
};
use opentalk_signaling_core::{
    ChunkFormat, DestroyContext, Event, InitContext, ModuleContext, ObjectStorage, Participant,
    SerdeJsonSnafu, SignalingModule, SignalingModuleError, SignalingModuleInitData,
    SignalingRoomId, VolatileStorage,
    assets::{NewAssetFileName, save_asset},
    control::{
        self,
        storage::{ControlStorageParticipantAttributes, LocalRoomAttributeId, USER_ID},
    },
};
use opentalk_types_common::{
    assets::FileExtension,
    modules::ModuleId,
    tenants::TenantId,
    time::Timestamp,
    users::{DisplayName, UserId},
};
use opentalk_types_signaling::{ParticipantId, Role};
use opentalk_types_signaling_legal_vote::{
    MODULE_ID,
    cancel::{CancelReason, CustomCancelReason},
    command::{Cancel, LegalVoteCommand, Stop, Vote},
    event::{
        Canceled, ErrorKind, FinalResults, LegalVoteEvent, PdfAsset, ReportedIssue, Response,
        Results, StopKind, Stopped, VoteFailed, VoteResponse, VoteResults, VoteSuccess,
        VotingRecord,
    },
    invalid::Invalid,
    parameters::Parameters,
    state::LegalVoteState,
    tally::Tally,
    token::Token,
    user_parameters::UserParameters,
    vote::{LegalVoteId, VoteKind, VoteOption},
};
use snafu::ResultExt;
use storage::{LegalVoteStorage, VoteScriptResult, VoteStatus};
use tokio::time::sleep;

use crate::{
    protocol::{RawProtocol, load_from_history},
    storage::protocol::{self as db_protocol},
};

mod error;
mod protocol;
mod report;

pub mod exchange;
pub mod storage;

/// A TimerEvent used for the vote expiration feature
pub struct TimerEvent {
    legal_vote_id: LegalVoteId,
}

trait LegalVoteStorageProvider {
    fn storage(&mut self) -> &mut dyn LegalVoteStorage;
}

impl LegalVoteStorageProvider for VolatileStorage {
    fn storage(&mut self) -> &mut dyn LegalVoteStorage {
        match self.as_mut() {
            Either::Left(v) => v,
            Either::Right(v) => v,
        }
    }
}

/// The legal vote [`SignalingModule`]
///
/// Holds a database interface and information about the underlying user & room. Vote information is
/// saved and managed in redis via the private `storage` module.
pub struct LegalVote {
    db: Arc<Db>,
    storage: Arc<ObjectStorage>,
    authz: Arc<Authz>,
    participant_id: ParticipantId,
    user_id: UserId,
    tenant_id: TenantId,
    room_id: SignalingRoomId,
}

#[async_trait::async_trait(?Send)]
impl SignalingModule for LegalVote {
    const NAMESPACE: ModuleId = MODULE_ID;

    type Params = ();

    type Incoming = LegalVoteCommand;
    type Outgoing = LegalVoteEvent;
    type ExchangeMessage = exchange::Event;

    type ExtEvent = TimerEvent;
    type FrontendData = LegalVoteState;
    type PeerFrontendData = ();

    async fn init(
        ctx: InitContext<'_, Self>,
        _params: &Self::Params,
        _protocol: &'static str,
    ) -> Result<Option<Self>, SignalingModuleError> {
        if let Participant::User(user) = ctx.participant() {
            Ok(Some(Self {
                db: ctx.db().clone(),
                storage: ctx.storage().clone(),
                authz: ctx.authz().clone(),
                participant_id: ctx.participant_id(),
                user_id: user.id,
                tenant_id: user.tenant_id,
                room_id: ctx.room_id(),
            }))
        } else {
            Ok(None)
        }
    }

    async fn on_event(
        &mut self,
        mut ctx: ModuleContext<'_, Self>,
        event: Event<'_, Self>,
    ) -> Result<(), SignalingModuleError> {
        match event {
            Event::Joined { frontend_data, .. } => {
                let current_vote = ctx
                    .volatile
                    .storage()
                    .current_vote_get(self.room_id)
                    .await?;

                *frontend_data = Some(
                    load_from_history(ctx.volatile.clone(), self.room_id, current_vote).await?,
                );

                let parameters = match current_vote {
                    Some(vote_id) => {
                        ctx.volatile
                            .storage()
                            .parameter_get(self.room_id, vote_id)
                            .await?
                    }
                    None => return Ok(()),
                };

                let Some(parameters) = parameters else {
                    return Ok(());
                };

                if parameters
                    .allowed_users
                    .is_some_and(|users| users.contains(&self.user_id))
                {
                    let entry = if parameters.inner.kind.is_hidden() {
                        db_protocol::v1::ProtocolEntry::new(db_protocol::v1::VoteEvent::UserJoined(
                            None.into(),
                        ))
                    } else {
                        db_protocol::v1::ProtocolEntry::new(db_protocol::v1::VoteEvent::UserJoined(
                            Some(db_protocol::v1::UserInfo {
                                issuer: self.user_id,
                                participant_id: self.participant_id,
                            })
                            .into(),
                        ))
                    };

                    ctx.volatile
                        .storage()
                        .protocol_add_entry(self.room_id, parameters.legal_vote_id, entry)
                        .await?;
                }
            }
            Event::Leaving => {
                if let Err(error) = self.handle_leaving(&mut ctx).await {
                    self.handle_error(&mut ctx, error)?;
                }
            }

            Event::WsMessage(msg) => {
                if let Err(error) = self.handle_ws_message(&mut ctx, msg).await {
                    self.handle_error(&mut ctx, error)?;
                }
            }
            Event::Exchange(event) => {
                if let Err(error) = self.handle_exchange_message(&mut ctx, event).await {
                    self.handle_error(&mut ctx, error)?;
                }
            }
            Event::Ext(timer_event) => {
                let vote_status = ctx
                    .volatile
                    .storage()
                    .get_vote_status(self.room_id, timer_event.legal_vote_id)
                    .await?;

                match vote_status {
                    VoteStatus::Active => {
                        let stop_kind = StopKind::Expired;

                        let expired_entry = db_protocol::v1::ProtocolEntry::new(
                            db_protocol::v1::VoteEvent::Stop(db_protocol::v1::StopKind::Expired),
                        );

                        if let Err(error) = self
                            .end_vote(
                                &mut ctx,
                                timer_event.legal_vote_id,
                                expired_entry,
                                stop_kind,
                            )
                            .await
                        {
                            match error {
                                LegalVoteError::Vote { source } => {
                                    log::error!("Failed to stop expired vote, error: {source}")
                                }
                                fatal @ LegalVoteError::Fatal { .. } => {
                                    self.handle_fatal_error(&mut ctx, fatal)?;
                                }
                            }
                        }
                    }
                    VoteStatus::Complete => {
                        // vote got stopped manually already, nothing to do
                    }
                    VoteStatus::Unknown => {
                        log::warn!("Legal vote timer contains an unknown vote id");
                    }
                }
            }

            // ignored events
            Event::RaiseHand
            | Event::LowerHand
            | Event::ParticipantJoined(_, _)
            | Event::ParticipantLeft(_)
            | Event::ParticipantUpdated(_, _)
            | Event::RoleUpdated(_) => (),
        }

        Ok(())
    }

    async fn on_destroy(self, ctx: DestroyContext<'_>) {
        let mut volatile = ctx.volatile.clone();
        let storage = volatile.storage();

        if ctx.destroy_room() {
            match storage.current_vote_get(self.room_id).await {
                Ok(current_vote_id) => {
                    if let Some(current_vote_id) = current_vote_id {
                        match self
                            .cancel_vote_unchecked(
                                storage,
                                current_vote_id,
                                CancelReason::RoomDestroyed,
                            )
                            .await
                        {
                            Ok(_) => {
                                if let Err(e) = self
                                    .save_protocol_in_database(storage, current_vote_id)
                                    .await
                                {
                                    log::error!("failed to save protocol to db {:?}", e)
                                }
                            }
                            Err(e) => log::error!(
                                "Failed to cancel active vote while destroying vote module {:?}",
                                e
                            ),
                        }
                    }
                }
                Err(e) => {
                    log::error!(
                        "Failed to get current vote id while destroying vote module, {:?}",
                        e
                    );
                }
            }

            if let Err(e) = self.cleanup_room(storage).await {
                log::error!("Failed to cleanup room on destroy, {:?}", e)
            }
        }
    }

    async fn build_params(
        _init: SignalingModuleInitData,
    ) -> Result<Option<Self::Params>, SignalingModuleError> {
        Ok(Some(()))
    }
}

impl LegalVote {
    /// Handle websocket messages send from the user
    async fn handle_ws_message(
        &mut self,
        ctx: &mut ModuleContext<'_, Self>,
        msg: LegalVoteCommand,
    ) -> Result<(), LegalVoteError> {
        let mut volatile = ctx.volatile.clone();
        let storage = volatile.storage();

        match msg {
            LegalVoteCommand::Start(incoming_parameters) => {
                if !matches!(ctx.role(), Role::Moderator) {
                    return Err(error::ErrorKind::InsufficientPermissions.into());
                }

                self.handle_start_message(ctx, incoming_parameters).await?;
            }
            LegalVoteCommand::Stop(Stop { legal_vote_id }) => {
                if !matches!(ctx.role(), Role::Moderator) {
                    return Err(error::ErrorKind::InsufficientPermissions.into());
                }

                self.stop_vote_routine(ctx, legal_vote_id).await?;
            }
            LegalVoteCommand::Cancel(Cancel {
                legal_vote_id,
                reason,
            }) => {
                if !matches!(ctx.role(), Role::Moderator) {
                    return Err(error::ErrorKind::InsufficientPermissions.into());
                }

                let entry = self
                    .cancel_vote(storage, legal_vote_id, reason.clone())
                    .await?;

                self.save_protocol_in_database(storage, legal_vote_id)
                    .await?;

                ctx.exchange_publish(
                    control::exchange::current_room_all_participants(self.room_id),
                    exchange::Event::Cancel(Canceled {
                        legal_vote_id,
                        reason: CancelReason::Custom(reason),
                        end_time: entry
                            .timestamp
                            .expect("Missing timestamp for cancel vote ProtocolEntry"),
                    }),
                );

                let parameters = ctx
                    .volatile
                    .storage()
                    .parameter_get(self.room_id, legal_vote_id)
                    .await?
                    .ok_or(error::ErrorKind::InvalidVoteId)?;

                if parameters.inner.create_pdf {
                    self.save_pdf(ctx, legal_vote_id, self.user_id, parameters.inner.timezone)
                        .await?
                }
            }
            LegalVoteCommand::Vote(vote_message) => {
                let (vote_response, auto_close) = self.cast_vote(ctx, vote_message).await?;

                if let Response::Success(VoteSuccess {
                    issuer,
                    vote_option,
                    consumed_token,
                }) = vote_response.response
                {
                    // Send a vote success message to all participants that have the same user id
                    ctx.exchange_publish(
                        control::exchange::current_room_by_user_id(self.room_id, self.user_id),
                        exchange::Event::Voted(exchange::VoteSuccess {
                            legal_vote_id: vote_message.legal_vote_id,
                            vote_option,
                            issuer,
                            consumed_token,
                        }),
                    );

                    let parameters = ctx
                        .volatile
                        .storage()
                        .parameter_get(self.room_id, vote_message.legal_vote_id)
                        .await?
                        .ok_or(error::ErrorKind::InvalidVoteId)?;
                    if parameters.inner.kind.is_live() {
                        let update = exchange::Event::Update(exchange::VoteUpdate {
                            legal_vote_id: vote_message.legal_vote_id,
                        });

                        ctx.exchange_publish(
                            control::exchange::current_room_all_participants(self.room_id),
                            update,
                        );
                    }

                    if auto_close {
                        let stop_kind = StopKind::Auto;

                        let auto_close_entry = db_protocol::v1::ProtocolEntry::new(
                            db_protocol::v1::VoteEvent::Stop(db_protocol::v1::StopKind::Auto),
                        );

                        self.end_vote(ctx, vote_message.legal_vote_id, auto_close_entry, stop_kind)
                            .await?;
                    }
                } else {
                    ctx.ws_send(LegalVoteEvent::Voted(vote_response));
                }
            }
            LegalVoteCommand::ReportIssue(report_issue) => {
                let parameters = ctx
                    .volatile
                    .storage()
                    .parameter_get(self.room_id, report_issue.legal_vote_id)
                    .await?
                    .ok_or(error::ErrorKind::InvalidVoteId)?;

                if !parameters
                    .allowed_users
                    .is_some_and(|users| users.contains(&self.user_id))
                {
                    return Err(error::ErrorKind::InsufficientPermissions.into());
                }

                let user_info = if parameters.inner.kind.is_hidden() {
                    None
                } else {
                    Some(db_protocol::v1::UserInfo {
                        issuer: self.user_id,
                        participant_id: self.participant_id,
                    })
                };

                let protocol_entry = db_protocol::v1::ProtocolEntry::new(
                    db_protocol::v1::VoteEvent::Issue(db_protocol::v1::ReportedIssue {
                        user_info,
                        issue: report_issue.issue.clone(),
                    }),
                );

                ctx.volatile
                    .storage()
                    .protocol_add_entry(self.room_id, report_issue.legal_vote_id, protocol_entry)
                    .await?;

                let reported_issue = ReportedIssue {
                    legal_vote_id: report_issue.legal_vote_id,
                    participant_id: user_info.map(|u| u.participant_id),
                    issue: report_issue.issue,
                };

                ctx.exchange_publish(
                    control::exchange::current_room_by_participant_id(
                        self.room_id,
                        parameters.initiator_id,
                    ),
                    exchange::Event::Issue(reported_issue),
                )
            }

            LegalVoteCommand::GeneratePdf(generate) => {
                if !matches!(ctx.role(), Role::Moderator) {
                    return Err(error::ErrorKind::InsufficientPermissions.into());
                }

                // check if the vote passed already
                if !ctx
                    .volatile
                    .storage()
                    .history_contains(self.room_id, generate.legal_vote_id)
                    .await?
                {
                    return Err(error::ErrorKind::InvalidVoteId.into());
                }

                self.save_pdf(ctx, generate.legal_vote_id, self.user_id, generate.timezone)
                    .await?;
            }
        }
        Ok(())
    }

    /// Handle incoming messages from the message exchange
    async fn handle_exchange_message(
        &mut self,
        ctx: &mut ModuleContext<'_, Self>,
        event: exchange::Event,
    ) -> Result<(), LegalVoteError> {
        match event {
            exchange::Event::Start(parameters) => {
                ctx.ws_send(LegalVoteEvent::Started(parameters));
            }
            exchange::Event::Stop(stopped) => {
                ctx.ws_send(LegalVoteEvent::Stopped(stopped));
            }
            exchange::Event::Voted(vote_success) => {
                ctx.ws_send(LegalVoteEvent::Voted(VoteResponse {
                    legal_vote_id: vote_success.legal_vote_id,
                    response: Response::Success(VoteSuccess {
                        vote_option: vote_success.vote_option,
                        issuer: vote_success.issuer,
                        consumed_token: vote_success.consumed_token,
                    }),
                }))
            }
            exchange::Event::Cancel(cancel) => {
                ctx.ws_send(LegalVoteEvent::Canceled(cancel));
            }
            exchange::Event::Update(update) => {
                let results = self
                    .get_vote_results(ctx.volatile.storage(), update.legal_vote_id)
                    .await?;

                ctx.ws_send(LegalVoteEvent::Updated(VoteResults {
                    legal_vote_id: update.legal_vote_id,
                    results,
                }));
            }
            exchange::Event::Issue(reported_issue) => {
                ctx.ws_send(LegalVoteEvent::ReportedIssue(reported_issue));
            }
            exchange::Event::FatalServerError => {
                ctx.ws_send(LegalVoteEvent::Error(ErrorKind::Internal));
            }
            exchange::Event::PdfAsset(pdf_asset) => {
                ctx.ws_send(LegalVoteEvent::PdfAsset(pdf_asset))
            }
        }
        Ok(())
    }

    async fn handle_start_message(
        &mut self,
        ctx: &mut ModuleContext<'_, LegalVote>,
        incoming_parameters: UserParameters,
    ) -> Result<(), LegalVoteError> {
        let legal_vote_id = self
            .new_vote_in_database()
            .await
            .whatever_context::<_, LegalVoteError>("Failed to create new vote in database")?;
        match self
            .start_vote_routine(ctx.volatile.storage(), legal_vote_id, incoming_parameters)
            .await
        {
            Ok((exchange_parameters, tokens)) => {
                self.grant_user_access(ctx, exchange_parameters.legal_vote_id)
                    .await?;

                if let Some(duration) = exchange_parameters.inner.duration {
                    ctx.add_event_stream(once(
                        sleep(duration.into()).map(move |_| TimerEvent { legal_vote_id }),
                    ));
                }

                for participant_id in ctx
                    .volatile
                    .storage()
                    .get_all_participants(self.room_id)
                    .await?
                {
                    let mut parameters = exchange_parameters.clone();
                    parameters.token = tokens.get(&participant_id).copied();
                    ctx.exchange_publish(
                        control::exchange::current_room_by_participant_id(
                            self.room_id,
                            participant_id,
                        ),
                        exchange::Event::Start(parameters),
                    );
                }
            }
            Err(start_error) => {
                log::warn!("Failed to start vote, {:?}", start_error);

                // return the cleanup error in case of failure as its more severe
                ctx.volatile
                    .storage()
                    .cleanup_vote(self.room_id, legal_vote_id)
                    .await?;

                return Err(start_error);
            }
        };
        Ok(())
    }

    /// Set all vote related redis keys
    async fn start_vote_routine(
        &self,
        storage: &mut dyn LegalVoteStorage,
        legal_vote_id: LegalVoteId,
        incoming_parameters: UserParameters,
    ) -> Result<(Parameters, HashMap<ParticipantId, Token>), LegalVoteError> {
        let start_time = Utc::now();

        let (max_votes, participant_tokens, allowed_users) = self
            .init_allowed_tokens(
                storage,
                legal_vote_id,
                &incoming_parameters.allowed_participants,
            )
            .await?;

        let parameters = Parameters {
            initiator_id: self.participant_id,
            legal_vote_id,
            start_time,
            max_votes,
            allowed_users: Some(allowed_users),
            inner: incoming_parameters,
            token: None,
        };

        storage
            .parameter_set(self.room_id, legal_vote_id, &parameters)
            .await?;

        self.init_vote_protocol(storage, legal_vote_id, start_time, parameters.clone())
            .await?;

        if !storage
            .current_vote_set(self.room_id, legal_vote_id)
            .await?
        {
            return Err(error::ErrorKind::VoteAlreadyActive.into());
        }

        Ok((parameters, participant_tokens))
    }

    /// Grant the room owner and legal-vote creator access to the legal-vote resource
    async fn grant_user_access(
        &self,
        ctx: &mut ModuleContext<'_, LegalVote>,
        legal_vote_id: LegalVoteId,
    ) -> Result<(), LegalVoteError> {
        let mut db_conn = self.db.get_conn().await?;

        let room_owner = Room::get(&mut db_conn, self.room_id.room_id())
            .await?
            .created_by;

        self.grant_module_resource_access(ctx, self.user_id, legal_vote_id)
            .await?;

        if self.user_id != room_owner {
            self.grant_module_resource_access(ctx, room_owner, legal_vote_id)
                .await?;
        }

        Ok(())
    }

    /// Grant the given user full access to the underlying module resource
    async fn grant_module_resource_access(
        &self,
        ctx: &mut ModuleContext<'_, LegalVote>,
        user_id: UserId,
        legal_vote_id: LegalVoteId,
    ) -> Result<(), LegalVoteError> {
        if let Err(e) = self
            .authz
            .grant_user_access(
                user_id,
                &[(
                    &legal_vote_id.inner().resource_id(),
                    &[AccessMethod::Get, AccessMethod::Put, AccessMethod::Delete],
                )],
            )
            .await
        {
            log::error!("Failed to add RBAC policy for legal vote: {}", e);
            ctx.volatile
                .storage()
                .cleanup_vote(self.room_id, legal_vote_id)
                .await?;

            return Err(error::ErrorKind::PermissionError.into());
        }

        Ok(())
    }

    /// Add the start entry to the protocol of this vote
    async fn init_vote_protocol(
        &self,
        storage: &mut dyn LegalVoteStorage,
        legal_vote_id: LegalVoteId,
        start_time: DateTime<Utc>,
        parameters: Parameters,
    ) -> Result<(), SignalingModuleError> {
        let start_entry = db_protocol::v1::ProtocolEntry::new_with_time(
            start_time,
            db_protocol::v1::VoteEvent::Start(db_protocol::v1::Start {
                issuer: self.user_id,
                parameters,
            }),
        );

        storage
            .protocol_add_entry(self.room_id, legal_vote_id, start_entry)
            .await?;

        Ok(())
    }

    /// Set the allowed users list for the provided `legal_vote_id` to its initial state
    ///
    /// Returns the maximum number of possible votes
    async fn init_allowed_tokens(
        &self,
        storage: &mut dyn LegalVoteStorage,
        legal_vote_id: LegalVoteId,
        allowed_participants: &[ParticipantId],
    ) -> Result<(u32, HashMap<ParticipantId, Token>, Vec<UserId>), LegalVoteError> {
        let mapped_users = storage
            .get_attribute_for_participants::<UserId>(
                allowed_participants,
                LocalRoomAttributeId {
                    room: self.room_id,
                    attribute: USER_ID,
                }
                .into(),
            )
            .await?;

        let mut invalid_participants = Vec::new();
        let mut user_tokens = HashMap::new();
        let mut participant_tokens = HashMap::new();
        let mut allowed_users = vec![];

        for (participant_id, maybe_user_id) in allowed_participants.iter().zip(mapped_users) {
            match maybe_user_id {
                Some(user_id) => {
                    let token = user_tokens.entry(user_id).or_insert_with(Token::generate);
                    participant_tokens.insert(*participant_id, *token);
                    allowed_users.push(user_id);
                }
                None => {
                    invalid_participants.push(*participant_id);
                }
            }
        }

        if !invalid_participants.is_empty() {
            return Err(LegalVoteError::Vote {
                source: error::ErrorKind::AllowlistContainsGuests {
                    guests: invalid_participants,
                },
            });
        }

        let max_votes = user_tokens.len();

        let tokens = user_tokens.values().copied().collect::<Vec<Token>>();
        storage
            .allow_token_set(self.room_id, legal_vote_id, tokens)
            .await?;

        Ok((max_votes as u32, participant_tokens, allowed_users))
    }

    /// Stop a vote
    ///
    /// Checks if the provided `legal_vote_id` is currently active before stopping the vote.
    ///
    /// Fails with `VoteError::InvalidVoteId` when the provided `legal_vote_id` does not match the active vote id.
    /// Adds a `ProtocolEntry` with `VoteEvent::Stop(Stop::UserStop(<user_id>))` to the vote protocol when successful.
    async fn stop_vote_routine(
        &self,
        ctx: &mut ModuleContext<'_, LegalVote>,
        legal_vote_id: LegalVoteId,
    ) -> Result<(), LegalVoteError> {
        if !self
            .is_current_vote_id(ctx.volatile.storage(), legal_vote_id)
            .await?
        {
            return Err(error::ErrorKind::InvalidVoteId.into());
        }

        let stop_kind = StopKind::ByParticipant(self.participant_id);

        let stop_entry = db_protocol::v1::ProtocolEntry::new(db_protocol::v1::VoteEvent::Stop(
            db_protocol::v1::StopKind::ByUser(self.user_id),
        ));

        self.end_vote(ctx, legal_vote_id, stop_entry, stop_kind)
            .await?;

        Ok(())
    }

    /// Cast a vote
    ///
    /// Checks if the provided `vote_message` contains valid values & calls [`storage::vote`].
    /// See [`storage::vote`] & [`storage::VOTE_SCRIPT`] for more details on the vote process.
    ///
    /// # Returns
    /// - Ok([`VoteResponse`], <should_auto_close>) in case of successfully executing [`storage::vote`].
    ///   Ok contains a tuple with the VoteResponse and a boolean that indicates that this was the last
    ///   vote needed and this vote can now be auto stopped. The boolean can only be true when this feature is enabled.
    /// - Err([`Error`]) in case of an redis error.
    async fn cast_vote(
        &self,
        ctx: &mut ModuleContext<'_, Self>,
        vote_message: Vote,
    ) -> Result<(VoteResponse, bool), LegalVoteError> {
        let storage = ctx.volatile.storage();

        match self
            .is_current_vote_id(storage, vote_message.legal_vote_id)
            .await
        {
            Ok(is_current) => {
                if !is_current {
                    return Ok((
                        VoteResponse {
                            legal_vote_id: vote_message.legal_vote_id,
                            response: Response::Failed(VoteFailed::InvalidVoteId),
                        },
                        false,
                    ));
                }
            }
            Err(error) => {
                if let LegalVoteError::Vote { source: _ } = error {
                    return Ok((
                        VoteResponse {
                            legal_vote_id: vote_message.legal_vote_id,
                            response: Response::Failed(VoteFailed::InvalidVoteId),
                        },
                        false,
                    ));
                } else {
                    return Err(error);
                }
            }
        }

        let parameters = match storage
            .parameter_get(self.room_id, vote_message.legal_vote_id)
            .await?
        {
            Some(parameters) => parameters,
            None => {
                return Ok((
                    VoteResponse {
                        legal_vote_id: vote_message.legal_vote_id,
                        response: Response::Failed(VoteFailed::InvalidVoteId),
                    },
                    false,
                ));
            }
        };

        if vote_message.option == VoteOption::Abstain && !parameters.inner.enable_abstain {
            return Ok((
                VoteResponse {
                    legal_vote_id: vote_message.legal_vote_id,
                    response: Response::Failed(VoteFailed::InvalidOption),
                },
                false,
            ));
        }

        let user_info = match parameters.inner.kind {
            VoteKind::Pseudonymous => None,
            VoteKind::RollCall | VoteKind::LiveRollCall => Some(db_protocol::v1::UserInfo {
                issuer: self.user_id,
                participant_id: self.participant_id,
            }),
        };

        let vote_event = db_protocol::v1::Vote {
            user_info,
            option: vote_message.option,
            token: vote_message.token,
        };

        let vote_result = storage
            .vote(self.room_id, vote_message.legal_vote_id, vote_event)
            .await?;

        let (response, should_auto_close) = match vote_result {
            VoteScriptResult::Success => (
                Response::Success(VoteSuccess {
                    vote_option: vote_message.option,
                    issuer: self.participant_id,
                    consumed_token: vote_message.token,
                }),
                false,
            ),
            VoteScriptResult::SuccessAutoClose => (
                Response::Success(VoteSuccess {
                    vote_option: vote_message.option,
                    issuer: self.participant_id,
                    consumed_token: vote_message.token,
                }),
                parameters.inner.auto_close,
            ),
            VoteScriptResult::InvalidVoteId => (Response::Failed(VoteFailed::InvalidVoteId), false),
            VoteScriptResult::Ineligible => (Response::Failed(VoteFailed::Ineligible), false),
        };

        Ok((
            VoteResponse {
                legal_vote_id: vote_message.legal_vote_id,
                response,
            },
            should_auto_close,
        ))
    }

    /// Check if the provided `legal_vote_id` equals the current active vote id
    ///
    /// Returns [`ErrorKind::NoVoteActive`] when no vote is active.
    async fn is_current_vote_id(
        &self,
        storage: &mut dyn LegalVoteStorage,
        legal_vote_id: LegalVoteId,
    ) -> Result<bool, LegalVoteError> {
        if let Some(current_legal_vote_id) = storage.current_vote_get(self.room_id).await? {
            Ok(current_legal_vote_id == legal_vote_id)
        } else {
            Err(error::ErrorKind::NoVoteActive.into())
        }
    }

    /// Cancel the active vote if the leaving participant is the initiator
    async fn handle_leaving(
        &self,
        ctx: &mut ModuleContext<'_, Self>,
    ) -> Result<(), LegalVoteError> {
        let storage = ctx.volatile.storage();

        let current_vote_id = match storage.current_vote_get(self.room_id).await? {
            Some(current_vote_id) => current_vote_id,
            None => return Ok(()),
        };

        let parameters = storage
            .parameter_get(self.room_id, current_vote_id)
            .await?
            .ok_or(error::ErrorKind::InvalidVoteId)?;

        if parameters
            .allowed_users
            .is_some_and(|users| users.contains(&self.user_id))
        {
            let entry = if parameters.inner.kind.is_hidden() {
                db_protocol::v1::ProtocolEntry::new(db_protocol::v1::VoteEvent::UserLeft(
                    None.into(),
                ))
            } else {
                db_protocol::v1::ProtocolEntry::new(db_protocol::v1::VoteEvent::UserLeft(
                    Some(db_protocol::v1::UserInfo {
                        issuer: self.user_id,
                        participant_id: self.participant_id,
                    })
                    .into(),
                ))
            };

            storage
                .protocol_add_entry(self.room_id, parameters.legal_vote_id, entry)
                .await?;
        }

        if parameters.initiator_id == self.participant_id {
            let reason = CancelReason::InitiatorLeft;

            let entry = self
                .cancel_vote_unchecked(storage, current_vote_id, reason.clone())
                .await?;

            self.save_protocol_in_database(storage, current_vote_id)
                .await?;

            ctx.exchange_publish(
                control::exchange::current_room_all_participants(self.room_id),
                exchange::Event::Cancel(Canceled {
                    legal_vote_id: current_vote_id,
                    reason,
                    end_time: entry
                        .timestamp
                        .expect("Missing timestamp on cancel vote ProtocolEntry"),
                }),
            );

            if parameters.inner.create_pdf {
                self.save_pdf(
                    ctx,
                    current_vote_id,
                    self.user_id,
                    parameters.inner.timezone,
                )
                .await?;
            }
        }

        Ok(())
    }

    /// Get the vote results for the specified `legal_vote_id`
    async fn get_vote_results(
        &self,
        storage: &mut dyn LegalVoteStorage,
        legal_vote_id: LegalVoteId,
    ) -> Result<Results, LegalVoteError> {
        let parameters = storage
            .parameter_get(self.room_id, legal_vote_id)
            .await?
            .ok_or(error::ErrorKind::InvalidVoteId)?;

        let tally = storage
            .count_get(self.room_id, legal_vote_id, parameters.inner.enable_abstain)
            .await?;

        let protocol_entries = storage.protocol_get(self.room_id, legal_vote_id).await?;
        let protocol = RawProtocol::from(&protocol_entries);

        let voting_record = (&protocol)
            .try_into()
            .map_err(|err| LegalVoteError::Fatal {
                message: "A error occurred while processing the protocol entries.".to_string(),
                source: Some(Box::new(err)),
            })?;

        Ok(Results {
            tally,
            voting_record,
        })
    }

    /// Cancel a vote
    ///
    /// Checks if the provided vote id is currently active before canceling the vote.
    async fn cancel_vote(
        &self,
        storage: &mut dyn LegalVoteStorage,
        legal_vote_id: LegalVoteId,
        reason: CustomCancelReason,
    ) -> Result<db_protocol::v1::ProtocolEntry, LegalVoteError> {
        if !self.is_current_vote_id(storage, legal_vote_id).await? {
            return Err(error::ErrorKind::InvalidVoteId.into());
        }

        self.cancel_vote_unchecked(storage, legal_vote_id, CancelReason::Custom(reason))
            .await
    }

    /// Cancel a vote without checking permissions
    ///
    /// Fails with `VoteError::InvalidVoteId` when the provided `legal_vote_id` does not match the active vote id.
    /// Adds a `ProtocolEntry` with `VoteEvent::Cancel` to the vote protocol when successful.
    async fn cancel_vote_unchecked(
        &self,
        storage: &mut dyn LegalVoteStorage,
        legal_vote_id: LegalVoteId,
        reason: CancelReason,
    ) -> Result<db_protocol::v1::ProtocolEntry, LegalVoteError> {
        let cancel_entry = db_protocol::v1::ProtocolEntry::new(db_protocol::v1::VoteEvent::Cancel(
            db_protocol::v1::Cancel {
                issuer: self.user_id,
                reason,
            },
        ));

        if !storage
            .end_current_vote(self.room_id, legal_vote_id, &cancel_entry)
            .await?
        {
            return Err(error::ErrorKind::InvalidVoteId.into());
        }

        Ok(cancel_entry)
    }

    /// End the vote behind `legal_vote_id` using the provided parameters as stop parameters
    async fn end_vote(
        &self,
        ctx: &mut ModuleContext<'_, Self>,
        legal_vote_id: LegalVoteId,
        end_entry: db_protocol::v1::ProtocolEntry,
        stop_kind: StopKind,
    ) -> Result<(), LegalVoteError> {
        let volatile = &mut ctx.volatile.clone();
        let storage = volatile.storage();
        if !storage
            .end_current_vote(self.room_id, legal_vote_id, &end_entry)
            .await?
        {
            return Err(error::ErrorKind::InvalidVoteId.into());
        }

        let final_results = self.validate_vote_results(ctx, legal_vote_id).await?;

        let final_results_entry = match &final_results {
            FinalResults::Valid(results) => {
                db_protocol::v1::ProtocolEntry::new(db_protocol::v1::VoteEvent::FinalResults(
                    db_protocol::v1::FinalResults::Valid(results.tally),
                ))
            }
            FinalResults::Invalid(invalid) => {
                db_protocol::v1::ProtocolEntry::new(db_protocol::v1::VoteEvent::FinalResults(
                    db_protocol::v1::FinalResults::Invalid(*invalid),
                ))
            }
        };

        storage
            .protocol_add_entry(self.room_id, legal_vote_id, final_results_entry)
            .await?;

        self.save_protocol_in_database(storage, legal_vote_id)
            .await?;

        ctx.exchange_publish(
            control::exchange::current_room_all_participants(self.room_id),
            exchange::Event::Stop(Stopped {
                legal_vote_id,
                kind: stop_kind,
                results: final_results,
                end_time: end_entry
                    .timestamp
                    .expect("Missing timestamp for end vote ProtocolEntry"),
            }),
        );

        let parameters = storage
            .parameter_get(self.room_id, legal_vote_id)
            .await?
            .ok_or(error::ErrorKind::InvalidVoteId)?;

        if parameters.inner.create_pdf {
            let timezone = parameters.inner.timezone;

            match stop_kind {
                // Send the pdf message to the participant id of the vote initiator in case of an auto stop
                StopKind::Auto => {
                    let protocol = storage.protocol_get(self.room_id, legal_vote_id).await?;

                    let pdf_asset = self
                        .create_pdf_asset(legal_vote_id, ctx.timestamp(), timezone, protocol)
                        .await?;

                    ctx.exchange_publish(
                        control::exchange::current_room_by_participant_id(
                            self.room_id,
                            parameters.initiator_id,
                        ),
                        exchange::Event::PdfAsset(pdf_asset),
                    );
                }
                _ => {
                    self.save_pdf(ctx, legal_vote_id, self.user_id, timezone)
                        .await?;
                }
            }
        }

        Ok(())
    }

    /// Checks if the vote results for `legal_vote_id` are equal to the protocols vote entries.
    ///
    /// Returns
    async fn validate_vote_results(
        &self,
        ctx: &mut ModuleContext<'_, Self>,
        legal_vote_id: LegalVoteId,
    ) -> Result<FinalResults, LegalVoteError> {
        let storage = ctx.volatile.storage();

        let parameters = storage
            .parameter_get(self.room_id, legal_vote_id)
            .await?
            .ok_or(error::ErrorKind::InvalidVoteId)?;

        let protocol_entries = storage.protocol_get(self.room_id, legal_vote_id).await?;
        let protocol = RawProtocol::from(&protocol_entries);

        let voting_record = match VotingRecord::try_from(&protocol) {
            Ok(voting_record) => voting_record,
            Err(err) => {
                return {
                    log::warn!(
                        "Something went wrong while generating `VotingRecord` out of `RawProtocol`. Error: {:?}",
                        err
                    );
                    Ok(FinalResults::Invalid(Invalid::ProtocolInconsistent))
                };
            }
        };

        let vote_options = voting_record.vote_option_list();

        let mut protocol_tally = Tally {
            yes: 0,
            no: 0,
            abstain: {
                if parameters.inner.enable_abstain {
                    Some(0)
                } else {
                    None
                }
            },
        };

        let mut total_votes = 0;

        for vote_option in &vote_options {
            total_votes += 1;

            match vote_option {
                VoteOption::Yes => protocol_tally.yes += 1,
                VoteOption::No => protocol_tally.no += 1,
                VoteOption::Abstain => {
                    if let Some(abstain) = &mut protocol_tally.abstain {
                        *abstain += 1;
                    } else {
                        return Ok(FinalResults::Invalid(Invalid::AbstainDisabled));
                    }
                }
            }
        }

        let tally = storage
            .count_get(self.room_id, legal_vote_id, parameters.inner.enable_abstain)
            .await?;

        if protocol_tally == tally && total_votes <= parameters.max_votes {
            Ok(FinalResults::Valid(Results {
                tally,
                voting_record,
            }))
        } else {
            Ok(FinalResults::Invalid(Invalid::VoteCountInconsistent))
        }
    }

    /// Remove the all vote related redis keys belonging to this room
    async fn cleanup_room(
        &self,
        storage: &mut dyn LegalVoteStorage,
    ) -> Result<(), SignalingModuleError> {
        let vote_history = storage.history_get(self.room_id).await?;

        for legal_vote_id in vote_history.iter() {
            storage.cleanup_vote(self.room_id, *legal_vote_id).await?
        }

        storage.history_delete(self.room_id).await?;

        if let Some(current_vote_id) = storage.current_vote_get(self.room_id).await? {
            storage.cleanup_vote(self.room_id, current_vote_id).await?;
            storage.current_vote_delete(self.room_id).await?;
        }

        Ok(())
    }

    /// Creates a new vote in the database
    ///
    /// Adds a new vote with an empty protocol to the database. Returns the [`VoteId`] of the new vote.
    async fn new_vote_in_database(&self) -> Result<LegalVoteId, SignalingModuleError> {
        let db = self.db.clone();

        let room_id = self.room_id.room_id();
        let tenant_id = self.tenant_id;

        let module_resource = NewModuleResource {
            tenant_id,
            room_id,
            created_by: self.user_id,
            namespace: Self::NAMESPACE.to_string(),
            tag: Some("protocol".into()),
            data: serde_json::to_value(db_protocol::NewProtocol::new(vec![])).unwrap(),
        }
        .insert(&mut db.get_conn().await?)
        .await?;

        Ok(LegalVoteId::from(module_resource.id))
    }

    /// Save the protocol for `legal_vote_id` in the database
    async fn save_protocol_in_database(
        &self,
        storage: &mut dyn LegalVoteStorage,
        legal_vote_id: LegalVoteId,
    ) -> Result<(), LegalVoteError> {
        let entries = storage.protocol_get(self.room_id, legal_vote_id).await?;

        let protocol = db_protocol::NewProtocol::new(entries);

        let db = self.db.clone();

        let mut conn = db.get_conn().await?;

        let protocol = serde_json::to_value(protocol).context(SerdeJsonSnafu {
            message: "Failed to serialize",
        })?;

        let add_protocol = opentalk_db_storage::module_resources::Operation::Add {
            path: "/".into(),
            value: protocol,
        };

        ModuleResource::patch(
            &mut conn,
            Filter::new()
                .with_id(*legal_vote_id.inner())
                .with_namespace("legal_vote".into()),
            vec![add_protocol],
        )
        .await
        .whatever_context::<_, LegalVoteError>("Failed to save protocol to database")?;

        Ok(())
    }

    /// Save the legal vote protocol as PDF
    ///
    /// Sends the [`Event::PdfAsset`] to the provided `msg_target`
    async fn save_pdf(
        &self,
        ctx: &mut ModuleContext<'_, Self>,
        legal_vote_id: LegalVoteId,
        msg_target: UserId,
        timezone: Option<chrono_tz::Tz>,
    ) -> Result<(), LegalVoteError> {
        let timezone = match timezone {
            Some(timezone) => Some(timezone),
            None => {
                let mut db_conn = self.db.get_conn().await?;
                let moderator = User::get(&mut db_conn, self.user_id).await?;
                moderator.timezone.map(Tz::from)
            }
        };

        let protocol = ctx
            .volatile
            .storage()
            .protocol_get(self.room_id, legal_vote_id)
            .await?;

        let pdf_asset = match self
            .create_pdf_asset(legal_vote_id, ctx.timestamp(), timezone, protocol)
            .await
        {
            Ok(pdf_asset) => pdf_asset,
            Err(LegalVoteError::Vote {
                source: error::ErrorKind::StorageExceeded,
            }) => {
                ctx.ws_send(LegalVoteEvent::Error(ErrorKind::StorageExceeded));
                return Ok(());
            }
            Err(e) => {
                log::error!("Failed to save legal vote asset: {e}");
                return Err(e);
            }
        };

        ctx.exchange_publish(
            control::exchange::current_room_by_user_id(self.room_id, msg_target),
            exchange::Event::PdfAsset(pdf_asset),
        );

        Ok(())
    }

    async fn get_referenced_user_names(
        &self,
        protocol: &[db_protocol::v1::ProtocolEntry],
    ) -> Result<BTreeMap<UserId, DisplayName>, LegalVoteError> {
        let mut conn = self.db.get_conn().await?;

        let user_ids = protocol
            .iter()
            .flat_map(db_protocol::v1::ProtocolEntry::get_referenced_user_ids)
            .collect::<BTreeSet<UserId>>();

        let user_names =
            opentalk_db_storage::users::User::get_all_by_ids(&mut conn, &Vec::from_iter(user_ids))
                .await?
                .into_iter()
                .map(|u| (u.id, u.display_name))
                .collect();

        Ok(user_names)
    }

    async fn create_pdf_asset(
        &self,
        legal_vote_id: LegalVoteId,
        timestamp: Timestamp,
        timezone: Option<chrono_tz::Tz>,
        protocol: Vec<db_protocol::v1::ProtocolEntry>,
    ) -> Result<PdfAsset, LegalVoteError> {
        let timezone = timezone.unwrap_or(chrono_tz::UTC);
        let user_names = self.get_referenced_user_names(&protocol).await?;

        let pdf_data = report::generate(
            user_names,
            protocol,
            &timezone,
            Path::new(&format!("{MODULE_ID}/{timestamp}")),
        )
        .whatever_context::<_, LegalVoteError>("Failed to generate legal vote PDF")?;

        // convert the data to a applicable type for the `save_asset()` fn
        let data: tokio_stream::Once<std::result::Result<bytes::Bytes, LegalVoteError>> =
            tokio_stream::once(Ok(Bytes::from(pdf_data)));

        let kind = "vote_protocol"
            .parse()
            .expect("Must be parseable as AssetFileKind");
        let filename = NewAssetFileName::new(kind, timestamp, FileExtension::pdf());

        let (asset_id, filename) = save_asset(
            &self.storage,
            self.db.clone(),
            self.room_id.room_id(),
            Some(Self::NAMESPACE),
            filename,
            data,
            ChunkFormat::Data,
        )
        .await?;

        Ok(PdfAsset {
            filename,
            legal_vote_id,
            asset_id,
        })
    }

    /// Check the provided `error` and handles the error cases
    fn handle_error(
        &self,
        ctx: &mut ModuleContext<'_, Self>,
        error: LegalVoteError,
    ) -> Result<(), LegalVoteError> {
        match error {
            LegalVoteError::Vote { source: error_kind } => {
                log::debug!("Error in legal_vote module {error_kind:?}",);
                ctx.ws_send(LegalVoteEvent::Error(error_kind.into()));
                Ok(())
            }
            fatal @ LegalVoteError::Fatal { .. } => {
                // todo: redis errors should be handled by the controller and not this module
                self.handle_fatal_error(ctx, fatal)
            }
        }
    }

    /// Handle a fatal error
    ///
    /// Send a `FatalServerError` message to all participants and add context to the error
    fn handle_fatal_error(
        &self,
        ctx: &mut ModuleContext<'_, Self>,
        fatal: LegalVoteError,
    ) -> Result<(), LegalVoteError> {
        ctx.exchange_publish(
            control::exchange::current_room_all_participants(self.room_id),
            exchange::Event::FatalServerError,
        );

        Err(fatal)
    }
}
