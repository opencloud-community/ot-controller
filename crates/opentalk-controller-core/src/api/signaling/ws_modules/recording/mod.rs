// SPDX-FileCopyrightText: OpenTalk GmbH <mail@opentalk.eu>
//
// SPDX-License-Identifier: EUPL-1.2

use std::{
    collections::{BTreeMap, BTreeSet, HashMap},
    sync::Arc,
};

use either::Either;
use futures::{stream::once, FutureExt};
use lapin_pool::{RabbitMqChannel, RabbitMqPool};
use opentalk_database::Db;
use opentalk_db_storage::streaming_targets::RoomStreamingTargetRecord;
use opentalk_signaling_core::{
    control::{
        self,
        storage::{ControlStorageParticipantAttributes as _, RECORDING_CONSENT},
    },
    CleanupScope, DestroyContext, Event, InitContext, ModuleContext, SignalingModule,
    SignalingModuleError, SignalingModuleInitData, SignalingRoomId, VolatileStorage,
};
use opentalk_types_common::{features::FeatureId, streaming::StreamingTargetId};
use opentalk_types_signaling::{ParticipantId, Role};
use opentalk_types_signaling_recording::{
    command::{PauseStreaming, RecordingCommand, SetConsent, StartStreaming, StopStreaming},
    event::{Error, RecorderError, RecordingEvent},
    module_id,
    peer_state::RecordingPeerState,
    record_feature,
    state::RecordingState,
    stream_feature, StreamStatus, StreamTargetSecret, NAMESPACE,
};
use snafu::{Report, ResultExt, Snafu};
use tokio::time::Duration;

use self::storage::RecordingStorage;
use super::recording_service::{self, RecordingService};

pub(crate) mod exchange;
mod rabbitmq;
pub(crate) mod storage;

#[derive(Debug, Clone, Copy, PartialEq, Eq, PartialOrd, Ord)]
enum RecordingFeature {
    Record,
    Stream,
}

#[derive(Debug, Snafu)]
pub enum TryFromRecordingFeatureError {
    #[snafu(display("Unknown recording feature \"{found}\""))]
    UnknownRecordingFeature { found: FeatureId },
}

impl TryFrom<FeatureId> for RecordingFeature {
    type Error = TryFromRecordingFeatureError;

    fn try_from(value: FeatureId) -> Result<Self, Self::Error> {
        Self::try_from(&value)
    }
}

impl TryFrom<&FeatureId> for RecordingFeature {
    type Error = TryFromRecordingFeatureError;

    fn try_from(value: &FeatureId) -> Result<Self, Self::Error> {
        match value {
            v if v == &record_feature() => Ok(Self::Record),
            v if v == &stream_feature() => Ok(Self::Stream),
            v => UnknownRecordingFeatureSnafu { found: v.clone() }.fail(),
        }
    }
}

pub struct Recording {
    id: ParticipantId,
    room: SignalingRoomId,
    room_encryption_enabled: bool,
    params: RecordingParams,
    recorder_started: bool,

    enabled_features: BTreeSet<RecordingFeature>,

    /// Whether or not the current participant is the recorder
    db: Arc<Db>,

    /// RabbitMQ channel used to send the recording start command over
    rabbitmq_channel: RabbitMqChannel,
}

#[derive(Clone)]
pub struct RecordingParams {
    pub queue: String,
}

pub enum RecorderExtEvent {
    /// The timeout message
    Timeout(BTreeSet<StreamingTargetId>),
}

pub(super) trait RecordingStorageProvider {
    fn storage(&mut self) -> &mut dyn RecordingStorage;
}

impl RecordingStorageProvider for VolatileStorage {
    fn storage(&mut self) -> &mut dyn RecordingStorage {
        match self.as_mut() {
            Either::Left(v) => v,
            Either::Right(v) => v,
        }
    }
}

#[async_trait::async_trait(?Send)]
impl SignalingModule for Recording {
    const NAMESPACE: &'static str = NAMESPACE;

    type Params = (Arc<RabbitMqPool>, RecordingParams);

    type Incoming = RecordingCommand;
    type Outgoing = RecordingEvent;
    type ExchangeMessage = exchange::Message;

    type ExtEvent = RecorderExtEvent;

    type FrontendData = RecordingState;
    type PeerFrontendData = RecordingPeerState;

    async fn init(
        ctx: InitContext<'_, Self>,
        params: &Self::Params,
        _protocol: &'static str,
    ) -> Result<Option<Self>, SignalingModuleError> {
        let (rabbitmq_pool, params) = params;

        let rabbitmq_channel = rabbitmq_pool.create_channel().await?;

        let enabled_features = ctx
            .room_tariff
            .module_features(&module_id())
            .into_iter()
            .flatten()
            .filter_map(|feature| RecordingFeature::try_from(feature.clone()).ok())
            .collect();

        Ok(Some(Self {
            id: ctx.participant_id(),
            room: ctx.room_id(),
            room_encryption_enabled: ctx.room().e2e_encryption,
            params: params.clone(),
            enabled_features,
            db: ctx.db().clone(),
            rabbitmq_channel,
            recorder_started: false,
        }))
    }

    fn get_provided_features() -> BTreeSet<FeatureId> {
        BTreeSet::from_iter([record_feature(), stream_feature()])
    }

    async fn on_event(
        &mut self,
        mut ctx: ModuleContext<'_, Self>,
        event: Event<'_, Self>,
    ) -> Result<(), SignalingModuleError> {
        match event {
            Event::Joined {
                control_data: _,
                frontend_data,
                participants,
            } => {
                self.handle_joined_event(ctx, frontend_data, participants)
                    .await?
            }
            Event::Leaving => {
                ctx.volatile
                    .storage()
                    .remove_attribute(self.room, self.id, RECORDING_CONSENT)
                    .await?;
            }
            Event::RaiseHand => {}
            Event::LowerHand => {}
            Event::ParticipantLeft(_) => {}
            Event::ParticipantJoined(id, data) | Event::ParticipantUpdated(id, data) => {
                let consent: Option<bool> = ctx
                    .volatile
                    .storage()
                    .get_attribute(self.room, id, RECORDING_CONSENT)
                    .await?;

                if let Some(consent) = consent {
                    *data = Some(RecordingPeerState {
                        consents_recording: consent,
                    })
                }
            }
            Event::RoleUpdated(_) => {}
            // Messages from frontend (Command)
            Event::WsMessage(msg) => match msg {
                RecordingCommand::SetConsent(SetConsent { consent }) => {
                    ctx.volatile
                        .storage()
                        .set_attribute(self.room, self.id, RECORDING_CONSENT, consent)
                        .await?;

                    ctx.invalidate_data();
                }
                RecordingCommand::StartStream(StartStreaming { target_ids }) => {
                    self.handle_start_streams(&mut ctx, target_ids).await?
                }
                RecordingCommand::PauseStream(PauseStreaming { target_ids }) => {
                    self.handle_pause_streams(&mut ctx, target_ids).await?
                }
                RecordingCommand::StopStream(StopStreaming { target_ids }) => {
                    self.handle_stop_streams(&mut ctx, target_ids).await?
                }
            },
            // Messages from other controllers, but they should land in the `recording_service` module
            Event::Exchange(msg) => match msg {
                exchange::Message::StreamUpdated(stream_updated) => {
                    ctx.ws_send(stream_updated);
                }
                exchange::Message::RecorderStarting => {
                    self.recorder_started = true;
                }
                exchange::Message::RecorderStopping => {
                    self.recorder_started = false;
                }
            },
            Event::Ext(msg) => match msg {
                RecorderExtEvent::Timeout(ids) => {
                    if ids.is_empty() {
                        return Ok(());
                    }

                    if self.recorder_started {
                        return Ok(());
                    }

                    let streams = ctx.volatile.storage().get_streams(self.room).await?;

                    if streams
                        .iter()
                        .any(|(_, target)| target.status == StreamStatus::Active)
                    {
                        return Ok(());
                    }

                    let streams = streams
                        .into_iter()
                        .filter(|(id, _)| ids.contains(id))
                        .map(|(id, mut target)| {
                            target.status = StreamStatus::Inactive;
                            (id, target)
                        })
                        .collect();

                    ctx.volatile
                        .storage()
                        .set_streams(self.room, &streams)
                        .await?;

                    log::warn!("Recorder ran into a timeout!");
                    ctx.ws_send(RecorderError::Timeout);
                }
            },
        }

        Ok(())
    }

    async fn on_destroy(self, mut ctx: DestroyContext<'_>) {
        match ctx.cleanup_scope {
            CleanupScope::None => (),
            CleanupScope::Local => cleanup_streams(&mut ctx, self.room).await,
            CleanupScope::Global => {
                cleanup_streams(&mut ctx, self.room).await;
                if self.room.breakout_room_id().is_some() {
                    // cleanup streams for main room
                    cleanup_streams(&mut ctx, SignalingRoomId::new(self.room.room_id(), None)).await
                }
            }
        }
    }

    async fn build_params(
        init: SignalingModuleInitData,
    ) -> Result<Option<Self::Params>, SignalingModuleError> {
        if let Some(queue) = init
            .shared_settings
            .load_full()
            .rabbit_mq
            .recording_task_queue
            .clone()
        {
            Ok(Some((
                init.rabbitmq_pool.clone(),
                RecordingParams { queue },
            )))
        } else {
            Ok(None)
        }
    }
}

async fn cleanup_streams(ctx: &mut DestroyContext<'_>, signaling_room_id: SignalingRoomId) {
    if let Err(e) = ctx
        .volatile
        .storage()
        .delete_all_streams(signaling_room_id)
        .await
    {
        log::error!("failed to delete streams, {}", Report::from_error(e));
    }
}

impl Recording {
    async fn initialize_streaming(
        &self,
        storage: &mut dyn RecordingStorage,
    ) -> Result<(), SignalingModuleError> {
        let can_record = self.enabled_features.contains(&RecordingFeature::Record)
            && self.room.breakout_room_id().is_none()
            && !self.room_encryption_enabled;
        let can_stream = self.enabled_features.contains(&RecordingFeature::Stream)
            && !self.room_encryption_enabled;

        let stock_streams = can_record.then_some((
            StreamingTargetId::generate(),
            StreamTargetSecret::recording(),
        ));
        let streams = if self.room.breakout_room_id().is_some() || !can_stream {
            BTreeMap::from_iter(stock_streams)
        } else {
            let mut conn = self.db.get_conn().await?;

            let streaming_targets =
                RoomStreamingTargetRecord::get_all_for_room(&mut conn, self.room.room_id()).await?;
            stock_streams
                .into_iter()
                .map(Ok)
                .chain(streaming_targets.into_iter().map(|target| {
                    let id = target.id;
                    StreamTargetSecret::try_from(target)
                        .map(|stream_target_secret| (id, stream_target_secret))
                        .with_whatever_context::<_, _, SignalingModuleError>(|err| format!("{err}"))
                }))
                .collect::<Result<_, SignalingModuleError>>()?
        };

        storage.set_streams(self.room, &streams).await?;

        Ok(())
    }

    async fn handle_joined_event(
        &mut self,
        ctx: ModuleContext<'_, Self>,
        frontend_data: &mut Option<RecordingState>,
        participants: &mut HashMap<ParticipantId, Option<RecordingPeerState>>,
    ) -> Result<(), SignalingModuleError> {
        if !ctx
            .volatile
            .storage()
            .is_streaming_initialized(self.room)
            .await?
        {
            self.initialize_streaming(ctx.volatile.storage()).await?;
        }

        let streams_res = ctx.volatile.storage().get_streams(self.room).await?;
        *frontend_data = Some({
            RecordingState {
                targets: BTreeMap::from_iter(
                    streams_res
                        .into_iter()
                        .map(|(target_id, stream_target)| (target_id, stream_target.into())),
                ),
            }
        });

        self.collect_participants_consents(ctx.volatile.storage(), participants)
            .await?;

        Ok(())
    }

    async fn handle_start_streams(
        &mut self,
        ctx: &mut ModuleContext<'_, Self>,
        target_ids: BTreeSet<StreamingTargetId>,
    ) -> Result<(), SignalingModuleError> {
        if ctx.role() != Role::Moderator {
            ctx.ws_send(Error::InsufficientPermissions);
            return Ok(());
        }

        if !self
            .target_ids_exist(ctx.volatile.storage(), &target_ids)
            .await?
        {
            ctx.ws_send(Error::InvalidStreamingId);
            return Ok(());
        }

        let is_recorder_running = ctx
            .volatile
            .storage()
            .streams_contain_status(
                self.room,
                BTreeSet::from_iter([
                    StreamStatus::Active,
                    StreamStatus::Starting,
                    StreamStatus::Paused,
                ]),
            )
            .await?;

        ctx.volatile
            .storage()
            .update_streams_status(self.room, &target_ids, StreamStatus::Starting)
            .await?;

        if !is_recorder_running {
            self.rabbitmq_channel
                .basic_publish(
                    "",
                    &self.params.queue,
                    Default::default(),
                    &serde_json::to_vec(&rabbitmq::InitializeRecorder {
                        room: self.room.room_id(),
                        breakout: self.room.breakout_room_id(),
                    })
                    .with_whatever_context::<_, _, SignalingModuleError>(
                        |_| "failed to initialize streaming".to_string(),
                    )?,
                    Default::default(),
                )
                .await
                .with_whatever_context::<_, _, SignalingModuleError>(|err| format!("{err}"))?;

            ctx.add_event_stream(once(
                tokio::time::sleep(Duration::from_secs(5u64))
                    .map(move |_| RecorderExtEvent::Timeout(target_ids)),
            ));

            return Ok(());
        }

        ctx.exchange_publish_to_namespace(
            control::exchange::current_room_all_recorders(self.room),
            RecordingService::module_id(),
            recording_service::exchange::Message::StartStreams { target_ids },
        );

        Ok(())
    }

    async fn handle_pause_streams(
        &mut self,
        ctx: &mut ModuleContext<'_, Self>,
        target_ids: BTreeSet<StreamingTargetId>,
    ) -> Result<(), SignalingModuleError> {
        if ctx.role() != Role::Moderator {
            ctx.ws_send(Error::InsufficientPermissions);
            return Ok(());
        }

        if !self
            .target_ids_exist(ctx.volatile.storage(), &target_ids)
            .await?
        {
            ctx.ws_send(Error::InvalidStreamingId);
            return Ok(());
        }

        ctx.exchange_publish_to_namespace(
            control::exchange::current_room_all_recorders(self.room),
            RecordingService::module_id(),
            recording_service::exchange::Message::PauseStreams { target_ids },
        );

        Ok(())
    }

    async fn handle_stop_streams(
        &mut self,
        ctx: &mut ModuleContext<'_, Self>,
        target_ids: BTreeSet<StreamingTargetId>,
    ) -> Result<(), SignalingModuleError> {
        if ctx.role() != Role::Moderator {
            ctx.ws_send(Error::InsufficientPermissions);
            return Ok(());
        }

        if !self
            .target_ids_exist(ctx.volatile.storage(), &target_ids)
            .await?
        {
            ctx.ws_send(Error::InvalidStreamingId);
            return Ok(());
        }

        let is_recorder_running = ctx
            .volatile
            .storage()
            .streams_contain_status(
                self.room,
                BTreeSet::from_iter([
                    StreamStatus::Active,
                    StreamStatus::Starting,
                    StreamStatus::Paused,
                ]),
            )
            .await;

        if let Ok(false) = is_recorder_running {
            ctx.ws_send(Error::RecorderNotStarted);
            return Ok(());
        }

        ctx.exchange_publish_to_namespace(
            control::exchange::current_room_all_recorders(self.room),
            RecordingService::module_id(),
            recording_service::exchange::Message::StopStreams { target_ids },
        );

        Ok(())
    }

    async fn collect_participants_consents(
        &self,
        storage: &mut dyn RecordingStorage,
        participants: &mut HashMap<ParticipantId, Option<RecordingPeerState>>,
    ) -> Result<(), SignalingModuleError> {
        let participant_ids: Vec<ParticipantId> = participants.keys().copied().collect();
        let participant_consents: Vec<Option<bool>> = storage
            .get_attribute_for_participants(self.room, &participant_ids, RECORDING_CONSENT)
            .await?;

        for (id, consent) in participant_ids.into_iter().zip(participant_consents) {
            if let Some(consent) = consent {
                participants.insert(
                    id,
                    Some(RecordingPeerState {
                        consents_recording: consent,
                    }),
                );
            }
        }

        Ok(())
    }

    async fn target_ids_exist(
        &self,
        storage: &mut dyn RecordingStorage,
        target_ids: &BTreeSet<StreamingTargetId>,
    ) -> Result<bool, SignalingModuleError> {
        for target_id in target_ids {
            if !storage.stream_exists(self.room, *target_id).await? {
                return Ok(false);
            }
        }
        Ok(true)
    }
}
