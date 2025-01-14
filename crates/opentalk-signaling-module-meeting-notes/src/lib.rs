// SPDX-FileCopyrightText: OpenTalk GmbH <mail@opentalk.eu>
//
// SPDX-License-Identifier: EUPL-1.2

use std::sync::Arc;

use chrono::{Duration, Utc};
use either::Either;
use exchange::GenerateUrl;
use opentalk_database::Db;
use opentalk_etherpad_client::EtherpadClient;
use opentalk_signaling_core::{
    assets::{save_asset, AssetError, NewAssetFileName},
    control::{
        self,
        storage::{ControlStorageParticipantAttributes as _, DISPLAY_NAME},
    },
    ChunkFormat, CleanupScope, DestroyContext, Event, InitContext, ModuleContext, ObjectStorage,
    SignalingModule, SignalingModuleError, SignalingModuleInitData, SignalingRoomId,
    VolatileStorage,
};
use opentalk_types_common::assets::FileExtension;
use opentalk_types_signaling::{ParticipantId, Role};
use opentalk_types_signaling_meeting_notes::{
    command::{MeetingNotesCommand, ParticipantSelection},
    event::{AccessUrl, Error, MeetingNotesEvent, PdfAsset},
    peer_state::MeetingNotesPeerState,
    NAMESPACE,
};
use redis_args::{FromRedisValue, ToRedisArgs};
use serde::{Deserialize, Serialize};
use snafu::{whatever, OptionExt, Report};
use storage::InitState;

use crate::storage::MeetingNotesStorage;

pub mod exchange;
pub mod storage;

const PAD_NAME: &str = "meeting_notes";

#[derive(Debug, Clone, PartialEq, Eq, Serialize, Deserialize, ToRedisArgs, FromRedisValue)]
#[to_redis_args(serde)]
#[from_redis_value(serde)]
struct SessionInfo {
    author_id: String,
    group_id: String,
    session_id: String,
    readonly: bool,
}

pub struct MeetingNotes {
    etherpad: EtherpadClient,
    participant_id: ParticipantId,
    room_id: SignalingRoomId,
    db: Arc<Db>,
    storage: Arc<ObjectStorage>,
}

trait MeetingNotesStorageProvide {
    fn storage(&mut self) -> &mut dyn MeetingNotesStorage;
}

impl MeetingNotesStorageProvide for VolatileStorage {
    fn storage(&mut self) -> &mut dyn MeetingNotesStorage {
        match self.as_mut() {
            Either::Left(v) => v,
            Either::Right(v) => v,
        }
    }
}

#[async_trait::async_trait(?Send)]
impl SignalingModule for MeetingNotes {
    const NAMESPACE: &'static str = NAMESPACE;
    type Params = opentalk_controller_settings::Etherpad;
    type Incoming = MeetingNotesCommand;
    type Outgoing = MeetingNotesEvent;
    type ExchangeMessage = exchange::Event;
    type ExtEvent = ();
    type FrontendData = ();
    type PeerFrontendData = MeetingNotesPeerState;

    async fn init(
        ctx: InitContext<'_, Self>,
        params: &Self::Params,
        _protocol: &'static str,
    ) -> Result<Option<Self>, SignalingModuleError> {
        let etherpad = EtherpadClient::new(params.url.clone(), params.api_key.clone());

        Ok(Some(Self {
            etherpad,
            participant_id: ctx.participant_id(),
            room_id: ctx.room_id(),
            db: ctx.db().clone(),
            storage: ctx.storage().clone(),
        }))
    }

    async fn on_event(
        &mut self,
        mut ctx: ModuleContext<'_, Self>,
        event: Event<'_, Self>,
    ) -> Result<(), SignalingModuleError> {
        match event {
            // Create a readonly session for every joining participant when the meeting-notes module is already initialized
            //
            // Send the current access state of every participant when the joining participant is a moderator
            Event::Joined {
                control_data: _,
                frontend_data: _,
                participants,
            } => {
                let state = ctx.volatile.storage().init_get(self.room_id).await?;

                if matches!(state, Some(state) if state == InitState::Initialized) {
                    let read_url = self.generate_url(&mut ctx, true).await?;

                    ctx.ws_send(MeetingNotesEvent::ReadUrl(AccessUrl { url: read_url }));

                    for (participant_id, access) in participants {
                        let session_info = ctx
                            .volatile
                            .storage()
                            .session_get(self.room_id, *participant_id)
                            .await?;

                        *access = session_info.map(|session_info| MeetingNotesPeerState {
                            readonly: session_info.readonly,
                        });
                    }
                }
            }
            Event::Leaving => {
                if let Some(session_info) = ctx
                    .volatile
                    .storage()
                    .session_delete(self.room_id, self.participant_id)
                    .await?
                {
                    self.etherpad
                        .delete_session(&session_info.session_id)
                        .await?
                }
            }
            Event::WsMessage(msg) => {
                self.on_ws_message(&mut ctx, msg).await?;
            }
            Event::Exchange(event) => {
                self.on_exchange_event(&mut ctx, event).await?;
            }
            Event::ParticipantUpdated(participant_id, peer_frontend_data)
            | Event::ParticipantJoined(participant_id, peer_frontend_data) => {
                let session_info = ctx
                    .volatile
                    .storage()
                    .session_get(self.room_id, participant_id)
                    .await?;

                *peer_frontend_data = session_info.map(|session_info| MeetingNotesPeerState {
                    readonly: session_info.readonly,
                });
            }
            _ => (),
        }

        Ok(())
    }

    async fn on_destroy(self, mut ctx: DestroyContext<'_>) {
        match ctx.cleanup_scope {
            CleanupScope::None => (),
            CleanupScope::Local => self.cleanup_room(&mut ctx, self.room_id).await,
            CleanupScope::Global => {
                self.cleanup_room(&mut ctx, self.room_id).await;

                if self.room_id.breakout_room_id().is_some() {
                    self.cleanup_room(&mut ctx, SignalingRoomId::new(self.room_id.room_id(), None))
                        .await
                }
            }
        }
    }

    async fn build_params(
        init: SignalingModuleInitData,
    ) -> Result<Option<Self::Params>, SignalingModuleError> {
        let etherpad = init.shared_settings.load_full().etherpad.clone();

        Ok(etherpad)
    }
}

impl MeetingNotes {
    async fn on_ws_message(
        &mut self,
        ctx: &mut ModuleContext<'_, Self>,
        msg: MeetingNotesCommand,
    ) -> Result<(), SignalingModuleError> {
        match msg {
            MeetingNotesCommand::SelectWriter(selection) => {
                if ctx.role() != Role::Moderator {
                    ctx.ws_send(Error::InsufficientPermissions);

                    return Ok(());
                }

                if !self
                    .verify_selection(ctx.volatile.storage(), &selection)
                    .await?
                {
                    ctx.ws_send(Error::InvalidParticipantSelection);
                }

                let targets = selection.participant_ids;

                // TODO: disallow selecting the same participant twice

                let storage = ctx.volatile.storage();

                let init_state = storage.try_start_init(self.room_id).await?;

                let first_init = match init_state {
                    Some(state) => {
                        match state {
                            InitState::Initializing => {
                                // Some other instance is currently initializing the etherpad
                                ctx.ws_send(Error::CurrentlyInitializing);
                                return Ok(());
                            }
                            InitState::Initialized => false,
                        }
                    }
                    None => {
                        // No init state was set before -> Initialize the etherpad in this module instance
                        if let Err(e) = self.init_etherpad(storage).await {
                            log::error!(
                                "Failed to init etherpad for room {}, {}",
                                self.room_id,
                                Report::from_error(e)
                            );

                            storage.init_delete(self.room_id).await?;

                            ctx.ws_send(Error::FailedInitialization);

                            return Ok(());
                        }

                        true
                    }
                };

                if first_init {
                    // all participants get access on first initialization
                    ctx.exchange_publish(
                        control::exchange::current_room_all_participants(self.room_id),
                        exchange::Event::GenerateUrl(GenerateUrl { writers: targets }),
                    );
                } else {
                    // calls after the first init only reach the targeted participants
                    for participant_id in targets {
                        ctx.exchange_publish(
                            control::exchange::current_room_by_participant_id(
                                self.room_id,
                                participant_id,
                            ),
                            exchange::Event::GenerateUrl(GenerateUrl {
                                writers: vec![participant_id],
                            }),
                        );
                    }
                }
            }
            MeetingNotesCommand::DeselectWriter(selection) => {
                if ctx.role() != Role::Moderator {
                    ctx.ws_send(Error::InsufficientPermissions);

                    return Ok(());
                }

                match ctx.volatile.storage().init_get(self.room_id).await? {
                    Some(state) => match state {
                        InitState::Initializing => {
                            ctx.ws_send(Error::CurrentlyInitializing);

                            return Ok(());
                        }
                        InitState::Initialized => (),
                    },
                    None => {
                        ctx.ws_send(Error::NotInitialized);

                        return Ok(());
                    }
                }

                if !self
                    .verify_selection(ctx.volatile.storage(), &selection)
                    .await?
                {
                    ctx.ws_send(Error::InvalidParticipantSelection);

                    return Ok(());
                }

                for participant_id in selection.participant_ids {
                    // check if its actually a writer
                    let session_info = ctx
                        .volatile
                        .storage()
                        .session_get(self.room_id, participant_id)
                        .await?;

                    let session_info = if let Some(session_info) = session_info {
                        session_info
                    } else {
                        continue;
                    };

                    // check if session is readonly already
                    if session_info.readonly {
                        continue;
                    }

                    // notify participant to recreate readonly sessions
                    ctx.exchange_publish(
                        control::exchange::current_room_by_participant_id(
                            self.room_id,
                            participant_id,
                        ),
                        exchange::Event::GenerateUrl(GenerateUrl { writers: vec![] }),
                    );
                }
            }
            MeetingNotesCommand::GeneratePdf => {
                if ctx.role() != Role::Moderator {
                    ctx.ws_send(Error::InsufficientPermissions);
                    return Ok(());
                }

                if !matches!(
                    ctx.volatile.storage().init_get(self.room_id).await?,
                    Some(InitState::Initialized)
                ) {
                    ctx.ws_send(Error::NotInitialized);
                    return Ok(());
                }

                let session_info = ctx
                    .volatile
                    .storage()
                    .session_get(self.room_id, self.participant_id)
                    .await?;
                if let Some(session_info) = session_info {
                    let group_id = ctx
                        .volatile
                        .storage()
                        .group_get(self.room_id)
                        .await?
                        .unwrap();

                    let pad_id = format!("{group_id}${PAD_NAME}");

                    let data = self
                        .etherpad
                        .download_pdf(&session_info.session_id, &pad_id)
                        .await?;

                    let kind = "meetingnotes_pdf"
                        .parse()
                        .expect("Must be parseable as AssetFileKind");

                    let filename =
                        NewAssetFileName::new(kind, ctx.timestamp(), FileExtension::pdf());

                    let (asset_id, filename) = match save_asset(
                        &self.storage,
                        self.db.clone(),
                        self.room_id.room_id(),
                        Some(Self::NAMESPACE),
                        filename,
                        data,
                        ChunkFormat::Data,
                    )
                    .await
                    {
                        Ok(asset_id) => asset_id,
                        Err(AssetError::AssetStorageExceeded) => {
                            ctx.ws_send(Error::StorageExceeded);
                            return Ok(());
                        }

                        Err(e) => {
                            let message = format!(
                                "Failed to save meetingnotes asset: {}",
                                Report::from_error(e)
                            );
                            log::error!("{message}");
                            whatever!("{message}");
                        }
                    };

                    ctx.exchange_publish(
                        control::exchange::current_room_all_participants(self.room_id),
                        exchange::Event::PdfAsset(PdfAsset { filename, asset_id }),
                    );
                } else {
                    ctx.ws_send(Error::NotInitialized);
                    return Ok(());
                }
            }
        }

        Ok(())
    }

    async fn on_exchange_event(
        &mut self,
        ctx: &mut ModuleContext<'_, Self>,
        event: exchange::Event,
    ) -> Result<(), SignalingModuleError> {
        match event {
            exchange::Event::GenerateUrl(GenerateUrl { writers }) => {
                if writers.contains(&self.participant_id) {
                    let write_url = self.generate_url(ctx, false).await?;

                    ctx.ws_send(MeetingNotesEvent::WriteUrl(AccessUrl { url: write_url }));
                } else {
                    let read_url = self.generate_url(ctx, true).await?;

                    ctx.ws_send(MeetingNotesEvent::ReadUrl(AccessUrl { url: read_url }));
                }

                ctx.invalidate_data();
            }
            exchange::Event::PdfAsset(pdf_asset) => ctx.ws_send(pdf_asset),
        }

        Ok(())
    }

    /// Initializes the etherpad-group and -pad for this room
    async fn init_etherpad(
        &self,
        storage: &mut dyn MeetingNotesStorage,
    ) -> Result<(), SignalingModuleError> {
        let group_id = self
            .etherpad
            .create_group_for(self.room_id.to_string())
            .await?;

        self.etherpad
            .create_group_pad(&group_id, PAD_NAME, None)
            .await?;

        storage.group_set(self.room_id, &group_id).await?;

        // flag this room as initialized
        storage.set_initialized(self.room_id).await?;

        Ok(())
    }

    /// Creates a new etherpad author for the participant
    ///
    /// Returns the generated author id
    async fn create_author(
        &self,
        storage: &mut dyn MeetingNotesStorage,
    ) -> Result<String, SignalingModuleError> {
        let display_name: String = storage
            .get_local_attribute(self.participant_id, self.room_id, DISPLAY_NAME)
            .await?
            .unwrap_or_default();

        let author_id = self
            .etherpad
            .create_author_if_not_exits_for(&display_name, &self.participant_id.to_string())
            .await?;

        Ok(author_id)
    }

    /// Creates a new etherpad session
    ///
    /// Returns the generated session id
    async fn create_session(
        &self,
        group_id: &str,
        author_id: &str,
        expire_duration: Duration,
        readonly: bool,
    ) -> Result<String, SignalingModuleError> {
        let expires = Utc::now()
            .checked_add_signed(expire_duration)
            .whatever_context::<&str, SignalingModuleError>("DateTime overflow")?
            .timestamp();

        let session_id = if readonly {
            self.etherpad
                .create_read_session(group_id, author_id, expires)
                .await?
        } else {
            self.etherpad
                .create_session(group_id, author_id, expires)
                .await?
        };

        Ok(session_id)
    }

    /// Creates a new author & session for a the participant
    ///
    /// Returns the `[SessionInfo]`
    async fn prepare_and_create_user_session(
        &mut self,
        storage: &mut dyn MeetingNotesStorage,
        readonly: bool,
    ) -> Result<SessionInfo, SignalingModuleError> {
        let author_id = self.create_author(storage).await?;

        let group_id = storage
            .group_get(self.room_id)
            .await?
            .whatever_context::<&str, SignalingModuleError>(
                "Missing group for room while preparing a new session",
            )?;

        // Currently there is no proper session refresh in etherpad. Due to the difficulty of setting new sessions
        // on the client across domains, we set the expire duration to 14 days and hope for the best.
        // Session refresh is merged and will be available in the next release: https://github.com/ether/etherpad-lite/pull/5361
        // TODO: use proper session refresh from etherpad once it's released
        let session_id = self
            .create_session(&group_id, &author_id, Duration::days(14), readonly)
            .await?;

        let session_info = SessionInfo {
            author_id,
            group_id,
            session_id,
            readonly,
        };

        storage
            .session_set(self.room_id, self.participant_id, &session_info)
            .await?;

        Ok(session_info)
    }

    /// Generates the auth-session url
    ///
    /// Creates a new user session which has ether write or read access, depending on the `readonly` flag.
    /// Any existing etherpad session for the participant will be removed and replaced.
    async fn generate_url(
        &mut self,
        ctx: &mut ModuleContext<'_, Self>,
        readonly: bool,
    ) -> Result<String, SignalingModuleError> {
        let storage = ctx.volatile.storage();

        // remove existing sessions from volatile storage
        if let Some(session_info) = storage
            .session_delete(self.room_id, self.participant_id)
            .await?
        {
            // If any exists, remove the participants session from the etherpad instance
            self.etherpad
                .delete_session(&session_info.session_id)
                .await?;
        }

        let session_info = self
            .prepare_and_create_user_session(storage, readonly)
            .await?;

        let url = self.etherpad.auth_session_url(
            &session_info.session_id,
            PAD_NAME,
            Some(&session_info.group_id),
        )?;

        // Notify other participants about the access change

        Ok(url.to_string())
    }

    /// Checks if all provided targets in a [`ParticipantSelection`] exist in this room
    ///
    /// Returns true when all targets are recognized
    async fn verify_selection(
        &self,
        storage: &mut dyn MeetingNotesStorage,
        selection: &ParticipantSelection,
    ) -> Result<bool, SignalingModuleError> {
        let room_participants = storage.get_all_participants(self.room_id).await?;

        Ok(selection
            .participant_ids
            .iter()
            .all(|target| room_participants.contains(target)))
    }

    async fn cleanup_room(&self, ctx: &mut DestroyContext<'_>, signaling_room_id: SignalingRoomId) {
        if let Err(e) = self
            .cleanup_etherpad(ctx.volatile.storage(), signaling_room_id)
            .await
        {
            log::error!(
                "Failed to cleanup etherpad for room {}: {}",
                signaling_room_id,
                e
            )
        }

        if let Err(e) = ctx.volatile.storage().cleanup(signaling_room_id).await {
            log::error!(
                "Failed to cleanup meeting-notes keys for room {} in volatile storage: {}",
                signaling_room_id,
                e
            );
        }
    }

    /// Removes the room related pad and group from etherpad
    async fn cleanup_etherpad(
        &self,
        storage: &mut dyn MeetingNotesStorage,
        signaling_room_id: SignalingRoomId,
    ) -> Result<(), SignalingModuleError> {
        let init_state = storage.init_get(signaling_room_id).await?;

        if init_state.is_none() {
            // Nothing to cleanup
            return Ok(());
        }

        let group_id = storage.group_get(signaling_room_id).await?.unwrap();

        let pad_id = format!("{group_id}${PAD_NAME}");

        self.etherpad.delete_pad(&pad_id).await?;

        // invalidate all sessions by deleting the group
        self.etherpad.delete_group(&group_id).await?;

        Ok(())
    }
}
