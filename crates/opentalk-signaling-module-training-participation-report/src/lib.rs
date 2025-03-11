// SPDX-FileCopyrightText: OpenTalk GmbH <mail@opentalk.eu>
//
// SPDX-License-Identifier: EUPL-1.2

//! Signaling module for tracking participant presence during a training session

#![deny(
    bad_style,
    missing_debug_implementations,
    missing_docs,
    overflowing_literals,
    patterns_in_fns_without_body,
    trivial_casts,
    trivial_numeric_casts,
    unsafe_code,
    unused,
    unused_extern_crates,
    unused_import_braces,
    unused_qualifications,
    unused_results
)]

use std::{
    collections::{BTreeMap, BTreeSet, HashMap},
    path::Path,
    sync::Arc,
};

use bytes::Bytes;
use chrono::{Duration, Local, Utc};
use chrono_tz::Tz;
use either::Either;
use futures::{stream::once, FutureExt as _};
use opentalk_database::Db;
use opentalk_signaling_core::{
    assets::{save_asset, AssetError, NewAssetFileName},
    control::{
        self,
        storage::{
            ControlStorage, ControlStorageParticipantAttributes as _, DISPLAY_NAME, IS_PRESENT,
            IS_ROOM_OWNER,
        },
        ControlStorageProvider,
    },
    ChunkFormat, DestroyContext, Event, InitContext, ModuleContext, ObjectStorage,
    ObjectStorageError, SignalingModule, SignalingModuleError, SignalingModuleInitData,
    VolatileStorage,
};
use opentalk_types_common::{
    assets::{asset_file_kind, AssetFileKind, FileExtension},
    events::{EventDescription, EventTitle},
    modules::ModuleId,
    rooms::RoomId,
    time::{TimeZone, Timestamp},
    users::{DisplayName, UserId},
};
use opentalk_types_signaling::ParticipantId;
use opentalk_types_signaling_control::state::ControlState;
use opentalk_types_signaling_training_participation_report::{
    command::TrainingParticipationReportCommand,
    event::{
        Error, PdfAsset, PresenceLoggingEnded, PresenceLoggingEndedReason, PresenceLoggingStarted,
        PresenceLoggingStartedReason, TrainingParticipationReportEvent,
    },
    state::{ParticipationLoggingState, TrainingParticipationReportState},
    TimeRange, MODULE_ID,
};
use rand::Rng as _;
use snafu::{Report, ResultExt as _};
use storage::{RoomState, TrainingParticipationReportStorage, TrainingReportState};
use template::ReportTemplateParameter;
use tokio::time::sleep;

pub mod exchange;
mod storage;
mod template;

const DEFAULT_TEMPLATE: &str = include_str!("training_participation_report.typ");

/// An event queued by the runner for itself to handle a timeout
#[derive(Debug, PartialEq, Eq)]
pub struct TimeoutEvent(u32);

/// Signaling module for tracking participant presence during a training session
#[derive(Debug)]
pub struct TrainingParticipationReport {
    room: RoomId,
    owner: UserId,
    participant: ParticipantId,
    db: Arc<Db>,
    storage: Arc<ObjectStorage>,
    room_owner_data: Option<RoomOwnerData>,
}

#[derive(Debug, Default)]
struct RoomOwnerData {
    trainees: BTreeSet<ParticipantId>,
    other_room_owners: BTreeSet<ParticipantId>,
    timeout_id: Option<u32>,
}

trait TrainingParticipationReportStorageProvider {
    fn storage(&mut self) -> &mut dyn TrainingParticipationReportStorage;
}

impl TrainingParticipationReportStorageProvider for VolatileStorage {
    fn storage(&mut self) -> &mut dyn TrainingParticipationReportStorage {
        match self.as_mut() {
            Either::Left(v) => v,
            Either::Right(v) => v,
        }
    }
}

#[async_trait::async_trait(?Send)]
impl SignalingModule for TrainingParticipationReport {
    const NAMESPACE: ModuleId = MODULE_ID;

    type Params = ();

    type Incoming = TrainingParticipationReportCommand;

    type Outgoing = TrainingParticipationReportEvent;

    type ExchangeMessage = exchange::Event;

    type ExtEvent = TimeoutEvent;

    type FrontendData = TrainingParticipationReportState;

    type PeerFrontendData = ();

    async fn init(
        ctx: InitContext<'_, Self>,
        _params: &Self::Params,
        _protocol: &'static str,
    ) -> Result<Option<Self>, SignalingModuleError> {
        Ok(Some(Self {
            room: ctx.room_id().room_id(),
            owner: ctx.room().created_by,
            participant: ctx.participant_id(),
            db: ctx.db().clone(),
            storage: ctx.storage().clone(),
            // The data required for the room owner instance of this module
            // is only available on join, so we will store it when the join
            // is handled.
            room_owner_data: None,
        }))
    }

    async fn on_event(
        &mut self,
        mut ctx: ModuleContext<'_, Self>,
        event: Event<'_, Self>,
    ) -> Result<(), SignalingModuleError> {
        match event {
            Event::Joined {
                control_data,
                participants,
                frontend_data,
            } => {
                self.handle_joined(&mut ctx, control_data, participants, frontend_data)
                    .await?;
            }
            Event::WsMessage(msg) => {
                self.handle_ws_message(&mut ctx, msg).await?;
            }
            Event::ParticipantJoined(participant, ..) => {
                self.handle_participant_joined(&mut ctx, participant)
                    .await?;
            }
            Event::ParticipantLeft(participant) => {
                self.handle_participant_left(&mut ctx, participant).await?
            }
            Event::Ext(TimeoutEvent(timeout_id)) => {
                self.handle_timeout(&mut ctx, timeout_id).await?
            }
            Event::Exchange(event) => self.handle_exchange_event(&mut ctx, event).await?,
            Event::Leaving => self.handle_leaving(&mut ctx).await?,
            Event::RaiseHand
            | Event::LowerHand
            | Event::ParticipantUpdated(_, _)
            | Event::RoleUpdated(_) => {}
        }

        Ok(())
    }

    async fn on_destroy(self, _ctx: DestroyContext<'_>) {}

    async fn build_params(
        _init: SignalingModuleInitData,
    ) -> Result<Option<Self::Params>, SignalingModuleError> {
        Ok(Some(()))
    }
}

impl TrainingParticipationReport {
    async fn handle_joined(
        &mut self,
        ctx: &mut ModuleContext<'_, Self>,
        control_data: &ControlState,
        participants: &HashMap<ParticipantId, Option<()>>,
        frontend_data: &mut Option<TrainingParticipationReportState>,
    ) -> Result<(), SignalingModuleError> {
        let mut state = ctx
            .volatile
            .storage()
            .get_recorded_presence_state(self.room, self.participant)
            .await?;

        if control_data.is_room_owner {
            let (other_room_owners, trainees) = self
                .load_already_present_room_owners_and_trainees(
                    participants,
                    ctx.volatile.control_storage(),
                )
                .await?;
            self.room_owner_data = Some(RoomOwnerData {
                other_room_owners,
                trainees,
                timeout_id: None,
            });
            if state == ParticipationLoggingState::WaitingForConfirmation {
                // Don't ask the room owner for confirmation
                state = ParticipationLoggingState::Enabled;
            }
        }
        *frontend_data = Some(TrainingParticipationReportState { state });
        Ok(())
    }

    async fn load_already_present_room_owners_and_trainees(
        &self,
        participants: &HashMap<ParticipantId, Option<()>>,
        control_storage: &mut dyn ControlStorage,
    ) -> Result<(BTreeSet<ParticipantId>, BTreeSet<ParticipantId>), SignalingModuleError> {
        let participants = Vec::from_iter(
            participants
                .keys()
                .filter(|k| **k != self.participant)
                .cloned(),
        );
        let is_present: Vec<Option<bool>> = control_storage
            .get_global_attribute_for_participants(&participants, self.room, IS_PRESENT)
            .await?;
        let is_room_owner: Vec<Option<bool>> = control_storage
            .get_global_attribute_for_participants(&participants, self.room, IS_ROOM_OWNER)
            .await?;
        let attributes = is_present
            .into_iter()
            .map(|p| p.unwrap_or_default())
            .zip(is_room_owner.into_iter().map(|p| p.unwrap_or_default()));
        let mut room_owners = BTreeSet::new();
        let mut trainees = BTreeSet::new();
        for (participant, is_room_owner) in participants.into_iter().zip(attributes).filter_map(
            |(participant, (is_present, is_room_owner))| {
                is_present.then_some((participant, is_room_owner))
            },
        ) {
            _ = if is_room_owner {
                &mut room_owners
            } else {
                &mut trainees
            }
            .insert(participant);
        }

        Ok((room_owners, trainees))
    }

    async fn handle_ws_message(
        &mut self,
        ctx: &mut ModuleContext<'_, Self>,
        msg: TrainingParticipationReportCommand,
    ) -> Result<(), SignalingModuleError> {
        match msg {
            TrainingParticipationReportCommand::EnablePresenceLogging {
                initial_checkpoint_delay,
                checkpoint_interval,
            } => {
                self.handle_command_enable_presence_logging(
                    ctx,
                    initial_checkpoint_delay,
                    checkpoint_interval,
                )
                .await
            }
            TrainingParticipationReportCommand::DisablePresenceLogging => {
                self.handle_command_disable_presence_logging(ctx).await
            }
            TrainingParticipationReportCommand::ConfirmPresence => {
                self.handle_command_confirm_presence(ctx).await
            }
        }
    }

    async fn handle_command_enable_presence_logging(
        &mut self,
        ctx: &mut ModuleContext<'_, Self>,
        initial_checkpoint_delay: Option<TimeRange>,
        checkpoint_interval: Option<TimeRange>,
    ) -> Result<(), SignalingModuleError> {
        let Some(room_owner_data) = self.room_owner_data.as_mut() else {
            ctx.ws_send(TrainingParticipationReportEvent::Error(
                Error::InsufficientPermissions,
            ));
            return Ok(());
        };
        let storage = ctx.volatile.storage();
        if storage
            .get_training_report_state(self.room)
            .await?
            .is_some()
        {
            ctx.ws_send(TrainingParticipationReportEvent::Error(
                Error::PresenceLoggingAlreadyEnabled,
            ));
            return Ok(());
        }
        const SECONDS_PER_MINUTE: u64 = 60;
        let initial_checkpoint_delay = initial_checkpoint_delay.unwrap_or(TimeRange {
            after: 10 * SECONDS_PER_MINUTE,
            within: 20 * SECONDS_PER_MINUTE,
        });
        let checkpoint_interval = checkpoint_interval.unwrap_or(TimeRange {
            after: (60 + 45) * SECONDS_PER_MINUTE,
            within: 30 * SECONDS_PER_MINUTE,
        });
        if room_owner_data.trainees.is_empty() {
            storage
                .initialize_room(
                    self.room,
                    ctx.timestamp,
                    TrainingReportState::WaitingForParticipant,
                    initial_checkpoint_delay,
                    checkpoint_interval,
                    room_owner_data.trainees.clone(),
                )
                .await?;

            ctx.exchange_publish(
                control::exchange::global_room_by_user_id(self.room, self.owner),
                exchange::Event::PresenceLoggingEnabled,
            );
            return Ok(());
        }

        storage
            .initialize_room(
                self.room,
                ctx.timestamp,
                TrainingReportState::WaitingForInitialTimeout,
                initial_checkpoint_delay.clone(),
                checkpoint_interval,
                room_owner_data.trainees.clone(),
            )
            .await?;
        ctx.exchange_publish(
            control::exchange::global_room_by_user_id(self.room, self.owner),
            exchange::Event::PresenceLoggingEnabled,
        );
        self.start_presence_logging(
            ctx,
            initial_checkpoint_delay,
            PresenceLoggingStartedReason::StartedManually,
        )
        .await
    }

    async fn start_presence_logging(
        &mut self,
        ctx: &mut ModuleContext<'_, Self>,
        initial_checkpoint_delay: TimeRange,
        reason: PresenceLoggingStartedReason,
    ) -> Result<(), SignalingModuleError> {
        let Some(room_owner_data) = self.room_owner_data.as_mut() else {
            unreachable!(
                "presence logging start can only be executed by the runner of a room owner"
            );
        };

        let first_checkpoint = Self::switch_to_next_checkpoint(
            room_owner_data,
            self.room,
            ctx,
            &initial_checkpoint_delay,
        )
        .await?;

        ctx.exchange_publish(
            control::exchange::global_room_all_participants(self.room),
            exchange::Event::PresenceLoggingStarted {
                first_checkpoint,
                reason,
            },
        );

        Ok(())
    }

    async fn switch_to_next_checkpoint(
        room_owner_data: &mut RoomOwnerData,
        room: RoomId,
        ctx: &mut ModuleContext<'_, Self>,
        time_range: &TimeRange,
    ) -> Result<Timestamp, SignalingModuleError> {
        let seconds_to_wait = Self::random_waiting_duration_seconds(time_range);

        let wait_duration = Duration::new(
            seconds_to_wait
                .try_into()
                .expect("value must not be greater than i64::MAXIMUM"),
            0,
        )
        .expect("value should be a valid duration");
        let checkpoint = ctx.timestamp + wait_duration;
        Self::start_checkpoint_timer(room_owner_data, ctx, checkpoint);

        ctx.volatile
            .storage()
            .switch_to_next_checkpoint(room, checkpoint)
            .await?;

        Ok(checkpoint)
    }

    fn start_checkpoint_timer(
        room_owner_data: &mut RoomOwnerData,
        ctx: &mut ModuleContext<'_, Self>,
        checkpoint: Timestamp,
    ) {
        let timeout_id = rand::rng().random();
        room_owner_data.timeout_id = Some(timeout_id);

        let duration = checkpoint
            .signed_duration_since(Utc::now())
            .to_std()
            .unwrap_or_default();
        let event = TimeoutEvent(timeout_id);

        ctx.add_event_stream(once(sleep(duration).map(move |_| event)));
    }

    async fn handle_command_disable_presence_logging(
        &mut self,
        ctx: &mut ModuleContext<'_, Self>,
    ) -> Result<(), SignalingModuleError> {
        if self.room_owner_data.is_none() {
            ctx.ws_send(TrainingParticipationReportEvent::Error(
                Error::InsufficientPermissions,
            ));
            return Ok(());
        }

        let Some(room_state) = ctx.volatile.storage().cleanup_room(self.room).await? else {
            ctx.ws_send(TrainingParticipationReportEvent::Error(
                Error::PresenceLoggingNotEnabled,
            ));
            return Ok(());
        };

        let report_state = room_state.report_state;

        if matches!(report_state, TrainingReportState::TrackingPresence) {
            self.create_training_participation_report(ctx, room_state)
                .await?;
        }

        if matches!(
            report_state,
            TrainingReportState::TrackingPresence | TrainingReportState::WaitingForInitialTimeout
        ) {
            let reason = PresenceLoggingEndedReason::StoppedManually;
            ctx.exchange_publish(
                control::exchange::global_room_all_participants(self.room),
                exchange::Event::PresenceLoggingEnded { reason },
            );
        }
        ctx.exchange_publish(
            control::exchange::global_room_by_user_id(self.room, self.owner),
            exchange::Event::PresenceLoggingDisabled,
        );
        Ok(())
    }

    async fn handle_command_confirm_presence(
        &mut self,
        ctx: &mut ModuleContext<'_, Self>,
    ) -> Result<(), SignalingModuleError> {
        let storage = ctx.volatile.storage();
        if storage.get_training_report_state(self.room).await?
            != Some(TrainingReportState::TrackingPresence)
        {
            ctx.ws_send(TrainingParticipationReportEvent::Error(
                Error::PresenceLoggingNotEnabled,
            ));
            return Ok(());
        }

        storage
            .record_presence_confirmation(self.room, self.participant, ctx.timestamp)
            .await?;
        ctx.ws_send(TrainingParticipationReportEvent::PresenceConfirmationLogged);

        Ok(())
    }

    async fn handle_participant_joined(
        &mut self,
        ctx: &mut ModuleContext<'_, Self>,
        participant: ParticipantId,
    ) -> Result<(), SignalingModuleError> {
        if let Some(room_owner_data) = self.room_owner_data.as_mut() {
            let is_room_owner: bool = ctx
                .volatile
                .control_storage()
                .get_global_attribute(participant, self.room, IS_ROOM_OWNER)
                .await?
                .unwrap_or_default();
            let state = ctx
                .volatile
                .storage()
                .get_training_report_state(self.room)
                .await?;

            let is_first_trainee = state == Some(TrainingReportState::WaitingForParticipant)
                && !is_room_owner
                && room_owner_data.trainees.is_empty();
            if is_room_owner {
                _ = room_owner_data.other_room_owners.insert(participant);
            } else {
                _ = room_owner_data.trainees.insert(participant);
                if state.is_some() {
                    ctx.volatile
                        .storage()
                        .add_known_participant(self.room, participant)
                        .await?;
                }
            }
            // This runner is only responsible if either no other room owner participants are present,
            // or if this participant has the lowest id among the room owners.
            let this_runner_is_responsible = match room_owner_data.other_room_owners.iter().next() {
                None => true,
                Some(other_room_owner) if other_room_owner > &self.participant => true,
                Some(_) => false,
            };
            if is_first_trainee && this_runner_is_responsible {
                let initial_checkpoint_delay = ctx
                    .volatile
                    .storage()
                    .get_initial_checkpoint_delay(self.room)
                    .await?;
                self.start_presence_logging(
                    ctx,
                    initial_checkpoint_delay,
                    PresenceLoggingStartedReason::FirstParticipantJoined,
                )
                .await?;
                ctx.volatile
                    .storage()
                    .set_training_report_state(
                        self.room,
                        TrainingReportState::WaitingForInitialTimeout,
                    )
                    .await?;
            }
        }
        Ok(())
    }

    async fn handle_participant_left(
        &mut self,
        ctx: &mut ModuleContext<'_, Self>,
        participant: ParticipantId,
    ) -> Result<(), SignalingModuleError> {
        let Some(room_owner_data) = self.room_owner_data.as_mut() else {
            return Ok(());
        };

        if room_owner_data.other_room_owners.remove(&participant) {
            // The leaving participant was another participant of the same room owner.
            // If anything has to be done, the leaving participant's runner will take care
            // of it and notify this runner with an exchange message.
            return Ok(());
        }

        if room_owner_data.trainees.remove(&participant) && !room_owner_data.trainees.is_empty() {
            // Some more trainees are present in the room.
            return Ok(());
        }

        // All trainee participants left, time to create the report if necessary.
        let Some(room_state) = ctx.volatile.storage().cleanup_room(self.room).await? else {
            return Ok(());
        };

        let reason = PresenceLoggingEndedReason::LastParticipantLeft;
        ctx.exchange_publish(
            control::exchange::global_room_by_user_id(self.room, self.owner),
            exchange::Event::PresenceLoggingEnded { reason },
        );
        let known_participants = room_owner_data.trainees.clone();

        if matches!(
            room_state.report_state,
            TrainingReportState::TrackingPresence
        ) {
            self.create_training_participation_report(ctx, room_state.clone())
                .await?;
        }

        let RoomState {
            start,
            initial_checkpoint_delay,
            checkpoint_interval,
            ..
        } = room_state;
        let report_state = TrainingReportState::WaitingForParticipant;

        ctx.volatile
            .storage()
            .initialize_room(
                self.room,
                start,
                report_state,
                initial_checkpoint_delay,
                checkpoint_interval,
                known_participants,
            )
            .await?;

        Ok(())
    }

    fn random_waiting_duration_seconds(range: &TimeRange) -> u64 {
        let mut rng = rand::rng();
        let offset = rng.random_range(0..range.within);
        const MAXIMUM_ALLOWED: u64 = i64::MAX as u64;
        range.after.saturating_add(offset).min(MAXIMUM_ALLOWED)
    }

    async fn handle_timeout(
        &mut self,
        ctx: &mut ModuleContext<'_, Self>,
        timeout_id: u32,
    ) -> Result<(), SignalingModuleError> {
        let Some(room_owner_data) = self.room_owner_data.as_mut() else {
            unreachable!("timeout is only created on by room owner");
        };
        if room_owner_data.timeout_id != Some(timeout_id) {
            // Timeout has been canceled or another timeout has been started
            // after the one we're currently handling, so this one is obsolete
            // and we just ignore it.
            return Ok(());
        }

        if matches!(
            ctx.volatile
                .storage()
                .get_training_report_state(self.room)
                .await?,
            None | Some(TrainingReportState::WaitingForParticipant),
        ) {
            // Presence logging has stopped, no need to handle the timer.
            return Ok(());
        }

        let time_range = ctx
            .volatile
            .storage()
            .get_checkpoint_interval(self.room)
            .await?;

        ctx.volatile
            .storage()
            .set_training_report_state(self.room, TrainingReportState::TrackingPresence)
            .await?;
        let _checkpoint =
            Self::switch_to_next_checkpoint(room_owner_data, self.room, ctx, &time_range).await?;

        ctx.exchange_publish(
            control::exchange::global_room_all_participants(self.room),
            exchange::Event::PresenceConfirmationRequested,
        );
        Ok(())
    }

    async fn handle_exchange_event(
        &mut self,
        ctx: &mut ModuleContext<'_, Self>,
        event: exchange::Event,
    ) -> Result<(), SignalingModuleError> {
        match event {
            exchange::Event::PresenceLoggingStarted {
                first_checkpoint,
                reason,
            } => {
                let message = if self.room_owner_data.is_none() {
                    PresenceLoggingStarted {
                        first_checkpoint: None,
                        reason: None,
                    }
                } else {
                    PresenceLoggingStarted {
                        first_checkpoint: Some(first_checkpoint),
                        reason: Some(reason),
                    }
                };
                ctx.ws_send(message);
                Ok(())
            }
            exchange::Event::PresenceLoggingEnded { reason } => {
                ctx.ws_send(PresenceLoggingEnded { reason });
                Ok(())
            }
            exchange::Event::PresenceLoggingEnabled => {
                ctx.ws_send(TrainingParticipationReportEvent::PresenceLoggingEnabled);
                Ok(())
            }
            exchange::Event::PresenceLoggingDisabled => {
                ctx.ws_send(TrainingParticipationReportEvent::PresenceLoggingDisabled);
                Ok(())
            }
            exchange::Event::PresenceConfirmationRequested => {
                if self.room_owner_data.is_none() {
                    ctx.ws_send(TrainingParticipationReportEvent::PresenceConfirmationRequested);
                }
                Ok(())
            }
            exchange::Event::RoomOwnerHandOver { next_checkpoint } => {
                let Some(room_owner_data) = self.room_owner_data.as_mut() else {
                    return Ok(());
                };
                Self::start_checkpoint_timer(room_owner_data, ctx, next_checkpoint);
                Ok(())
            }
            exchange::Event::PdfAsset(pdf_asset) => {
                ctx.ws_send(pdf_asset);
                Ok(())
            }
        }
    }

    async fn handle_leaving(
        &mut self,
        ctx: &mut ModuleContext<'_, Self>,
    ) -> Result<(), SignalingModuleError> {
        let Some(room_owner_data) = self.room_owner_data.as_ref() else {
            return Ok(());
        };

        match ctx
            .volatile
            .storage()
            .get_training_report_state(self.room)
            .await?
        {
            None => {
                return Ok(());
            }
            Some(TrainingReportState::WaitingForParticipant) => {
                return Ok(());
            }
            Some(TrainingReportState::WaitingForInitialTimeout)
            | Some(TrainingReportState::TrackingPresence) => {
                let other_room_owner = room_owner_data.other_room_owners.iter().next();

                match other_room_owner {
                    None => {
                        ctx.exchange_publish(
                            control::exchange::global_room_all_participants(self.room),
                            exchange::Event::PresenceLoggingEnded {
                                reason: PresenceLoggingEndedReason::CreatorLeft,
                            },
                        );

                        // All room owner participants, let's abort the waiting for initial timeout.
                        let Some(room_state) =
                            ctx.volatile.storage().cleanup_room(self.room).await?
                        else {
                            return Ok(());
                        };

                        if matches!(
                            room_state.report_state,
                            TrainingReportState::TrackingPresence
                        ) {
                            self.create_training_participation_report(ctx, room_state)
                                .await?;
                        }
                        return Ok(());
                    }
                    Some(room_owner_id) => {
                        // At least one other room owner participant is present.

                        if room_owner_data.timeout_id.is_none() {
                            // This room owner participant was not responsible for organizing the checkpoints.
                            return Ok(());
                        };

                        // Let's hand over the responsibility for checkpoint
                        // handling to the next runner.

                        let Some(next_checkpoint) = ctx
                            .volatile
                            .storage()
                            .get_next_checkpoint(self.room)
                            .await?
                        else {
                            return Ok(());
                        };

                        ctx.exchange_publish(
                            control::exchange::global_room_by_participant_id(
                                self.room,
                                *room_owner_id,
                            ),
                            exchange::Event::RoomOwnerHandOver { next_checkpoint },
                        );
                    }
                }
            }
        }
        Ok(())
    }

    async fn create_training_participation_report(
        &mut self,
        ctx: &mut ModuleContext<'_, Self>,
        room_state: RoomState,
    ) -> Result<(), SignalingModuleError> {
        let mut conn = self.db.get_conn().await?;
        let event = opentalk_db_storage::events::Event::get_for_room(&mut conn, self.room)
            .await?
            .ok_or(SignalingModuleError::NotFoundError {
                message: "Event for room not found".to_string(),
            })?;

        let timezone = event.starts_at_tz.unwrap_or(TimeZone::from(Tz::UTC));

        let required_participants = Vec::from_iter(room_state.known_participants.clone());

        let display_names: Vec<Option<DisplayName>> = ctx
            .volatile
            .control_storage()
            .get_global_attribute_for_participants(&required_participants, self.room, DISPLAY_NAME)
            .await?;
        let participants = required_participants
            .into_iter()
            .zip(display_names)
            .collect();

        let report = Self::generate_pdf_report(
            DEFAULT_TEMPLATE.to_string(),
            room_state,
            timezone,
            participants,
            event.title,
            event.description,
            ctx.timestamp,
        )
        .await
        .with_whatever_context::<_, _, SignalingModuleError>(|_| {
            ctx.ws_send(Error::Generate);
            "Failed to create pdf"
        })?;
        self.upload_pdf(report, ctx).await;

        Ok(())
    }

    async fn generate_pdf_report(
        template: String,
        room_state: RoomState,
        report_timezone: TimeZone,
        participants: BTreeMap<ParticipantId, Option<DisplayName>>,
        title: EventTitle,
        description: EventDescription,
        end: Timestamp,
    ) -> Result<Vec<u8>, SignalingModuleError> {
        let timestamp = Local::now().naive_local().format("%Y-%m-%dT%H:%M:%S.%f");
        let report_tz = Tz::from(report_timezone);

        Self::generate_pdf_report_from_template(
            template,
            &ReportTemplateParameter::build(
                &room_state,
                &report_tz,
                participants,
                title,
                description,
                end,
            ),
            Path::new(&format!("{MODULE_ID}/{timestamp}")),
        )
    }

    fn generate_pdf_report_from_template(
        template: String,
        parameter: &ReportTemplateParameter,
        dump_to_relative_path: &Path,
    ) -> Result<Vec<u8>, SignalingModuleError> {
        let dump_to_path = std::env::var("OPENTALK_REPORT_DUMP_PATH")
            .map(|p| Path::new(&p).join(dump_to_relative_path))
            .ok();

        let pdf = opentalk_report_generation::generate_pdf_report(
            template,
            BTreeMap::from_iter([(
                Path::new("data.json"),
                serde_json::to_string_pretty(parameter)
                    .unwrap()
                    .into_bytes()
                    .into(),
            )]),
            dump_to_path.as_deref(),
        )
        .whatever_context::<_, SignalingModuleError>("unable to build pdf")?;
        Ok(pdf)
    }

    async fn upload_pdf(&mut self, report: Vec<u8>, ctx: &mut ModuleContext<'_, Self>) {
        const ASSET_FILE_KIND: AssetFileKind = asset_file_kind!("training_participation_report");
        let file_name =
            NewAssetFileName::new(ASSET_FILE_KIND, Timestamp::now(), FileExtension::pdf());
        let report =
            async_stream::stream!(yield Result::<_, ObjectStorageError>::Ok(Bytes::from(report)));
        let report = Box::pin(report);
        let result = save_asset(
            &self.storage,
            self.db.clone(),
            self.room,
            Some(Self::NAMESPACE),
            file_name,
            report,
            ChunkFormat::Data,
        )
        .await;

        // If storing the asset failed, we report the error and silently return.
        let (asset_id, file_name) = match result {
            Ok(inner) => inner,
            Err(AssetError::AssetStorageExceeded) => {
                log::debug!("Storage exceeded while storing training participation report");
                ctx.ws_send(Error::StorageExceeded);
                return;
            }
            Err(e) => {
                log::error!(
                    "Error while storing attendance report: {}",
                    Report::from_error(e)
                );
                ctx.ws_send(Error::Storage);
                return;
            }
        };

        let pdf_asset = PdfAsset {
            filename: file_name,
            asset_id,
        };
        log::debug!("Generated meeting attendance report: {:?}", pdf_asset);
        ctx.exchange_publish(
            control::exchange::global_room_by_user_id(self.room, self.owner),
            exchange::Event::PdfAsset(pdf_asset.clone()),
        );
    }
}

#[cfg(test)]
mod tests {
    use std::path::Path;

    use insta::assert_snapshot;

    use crate::{
        template::ReportTemplateParameter, TrainingParticipationReport, DEFAULT_TEMPLATE, MODULE_ID,
    };

    fn generate(sample_name: &str, parameter: &ReportTemplateParameter) -> String {
        let pdf = TrainingParticipationReport::generate_pdf_report_from_template(
            DEFAULT_TEMPLATE.to_string(),
            parameter,
            Path::new(&format!("{MODULE_ID}/{sample_name}")),
        )
        .expect("generation should work");
        pdf_extract::extract_text_from_mem(&pdf)
            .expect("text should be extractable from generated pdf")
    }

    #[test]
    fn generate_report_small() {
        assert_snapshot!(
            generate(
                "small",
                &crate::template::tests::example_small()
            ),
            @r#"
        Training participation report
         Meeting: OpenTalk introduction training

        Description: —

        Report timezone: Europe/Berlin

        Training start: 2025-02-18 09:01

        Training end: 2025-02-18 13:32

        Participation checkpoints
         № Participant 09:22 11:22 13:19

        1 Bob Burton 09:22 11:25 —

        2 Charlie Cooper 09:22 11:25 13:19
        "#
        );
    }

    #[test]
    fn generate_report_medium() {
        assert_snapshot!(
            generate(
                "medium",
                &crate::template::tests::example_medium()
            ),
            @r#"
        Training participation report
         Meeting: OpenTalk introduction training

        Description: —

        Report timezone: Europe/Berlin

        Training start: 2025-02-18 09:01

        Training end: 2025-02-19 03:32

        Participation checkpoints
         № Participant 09:22 11:22 13:19 15:08 17:21 19:31 21:31 23:36

        1 Bob Burton 09:22 11:25 — — 17:21 19:31 21:31 —

        2 Charlie Cooper 09:22 — 13:19 — — — — 23:36

        № Participant 01:37 03:27

        1 Bob Burton 01:37 03:27

        2 Charlie Cooper 01:55 —
        "#
        );
    }

    #[test]
    fn generate_report_large() {
        assert_snapshot!(
            generate(
                "large",
                &crate::template::tests::example_large()
            ),
            @r#"
        Training participation report
         Meeting: OpenTalk introduction training

        Description: —

        Report timezone: Europe/Berlin

        Training start: 2025-02-18 09:01

        Training end: 2025-02-19 03:32

        Participation checkpoints
         № Participant 09:22 11:22 13:19 15:08 17:21 19:31 21:31 23:36

        1 Bob Burton 09:22 11:22 13:19 15:08 17:21 19:31 21:31 23:36

        2 Charlie Cooper 09:22 11:22 13:19 15:08 17:21 19:31 21:31 23:36

        3 Dave Dunn 09:22 11:22 13:19 15:08 17:21 19:31 21:31 23:36

        4 Erin Eaton 09:22 11:22 13:19 15:08 17:21 19:31 21:31 23:36

        5 Frank Floyd 09:22 11:22 13:19 15:08 17:21 19:31 21:31 23:36

        6 George Garvis 09:22 11:22 13:19 15:08 17:21 19:31 21:31 23:36

        7 Hannah Händl 09:22 11:22 13:19 15:08 17:21 19:31 21:31 23:36

        8 Isaac Ivens (Northwind Ltd.) 09:22 11:22 13:19 15:08 17:21 19:31 21:31 23:36

        9 Jack Jilbert 09:22 11:22 13:19 15:08 17:21 19:31 21:31 23:36

        10 Karl Keating 09:22 11:22 13:19 15:08 17:21 19:31 21:31 23:36

        11 Leann Larn 09:22 11:22 13:19 15:08 17:21 19:31 21:31 23:36

        12 Marlene M. Maine 09:22 11:22 13:19 15:08 17:21 19:31 21:31 23:36

        13 Neil Neugraten 09:22 11:22 13:19 15:08 17:21 19:31 21:31 23:36

        14 Ofelia Ollivander 09:22 11:22 13:19 15:08 17:21 19:31 21:31 23:36

        15 Patrick Peterson 09:22 11:22 13:19 15:08 17:21 19:31 21:31 23:36

        16 Quinton Quintana 09:22 11:22 13:19 15:08 17:21 19:31 21:31 23:36

        17 Roger Richard 09:22 11:22 13:19 15:08 17:21 19:31 21:31 23:36

        18 Sophie Stanton 09:22 11:22 13:19 15:08 17:21 19:31 21:31 23:36

        19 Thalia Tyler 09:22 11:22 13:19 15:08 17:21 19:31 21:31 23:36

        20 Ulises Underwood 09:22 11:22 13:19 15:08 17:21 19:31 21:31 23:36

        21 Valentina Villalobos 09:22 11:22 13:19 15:08 17:21 19:31 21:31 23:36

        22 Wallace Winters 09:22 11:22 13:19 15:08 17:21 19:31 21:31 23:36

        23 Xiomara Xiong 09:22 11:22 13:19 15:08 17:21 19:31 21:31 23:36

        24 Yousef Yu 09:22 11:22 13:19 15:08 17:21 19:31 21:31 23:36

        25 Zainab Zavala 09:22 11:22 13:19 15:08 17:21 19:31 21:31 23:36

        26 嬴嬴嬴 09:22 11:22 13:19 15:08 17:21 19:31 21:31 23:36

        № Participant 01:37 03:27

        1 Bob Burton 01:37 03:27

        2 Charlie Cooper 01:37 03:27

        3 Dave Dunn 01:37 03:27

        4 Erin Eaton 01:37 03:27

        5 Frank Floyd 01:37 03:27

        № Participant 01:37 03:27

        6 George Garvis 01:37 03:27

        7 Hannah Händl 01:37 03:27

        8 Isaac Ivens (Northwind Ltd.) 01:37 03:27

        9 Jack Jilbert 01:37 03:27

        10 Karl Keating 01:37 03:27

        11 Leann Larn 01:37 03:27

        12 Marlene M. Maine 01:37 03:27

        13 Neil Neugraten 01:37 03:27

        14 Ofelia Ollivander 01:37 03:27

        15 Patrick Peterson 01:37 03:27

        16 Quinton Quintana 01:37 03:27

        17 Roger Richard 01:37 03:27

        18 Sophie Stanton 01:37 03:27

        19 Thalia Tyler 01:37 03:27

        20 Ulises Underwood 01:37 03:27

        21 Valentina Villalobos 01:37 03:27

        22 Wallace Winters 01:37 03:27

        23 Xiomara Xiong 01:37 03:27

        24 Yousef Yu 01:37 03:27

        25 Zainab Zavala 01:37 03:27

        26 嬴嬴嬴 01:37 03:27
        "#
        );
    }
}
