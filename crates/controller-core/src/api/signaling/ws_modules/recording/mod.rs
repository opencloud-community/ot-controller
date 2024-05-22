// SPDX-FileCopyrightText: OpenTalk GmbH <mail@opentalk.eu>
//
// SPDX-License-Identifier: EUPL-1.2

use std::{
    collections::{BTreeMap, BTreeSet, HashMap},
    sync::Arc,
};

use futures::{stream::once, FutureExt};
use lapin_pool::{RabbitMqChannel, RabbitMqPool};
use opentalk_database::Db;
use opentalk_db_storage::streaming_targets::RoomStreamingTargetRecord;
use opentalk_signaling_core::{
    control::{self, storage::ControlStorage as _},
    DestroyContext, Event, InitContext, ModuleContext, RedisConnection, SignalingModule,
    SignalingModuleError, SignalingModuleInitData, SignalingRoomId,
};
use opentalk_types::{
    core::{ParticipantId, StreamingTargetId},
    signaling::{
        recording::{
            command::{self, RecordingCommand},
            event::{Error, RecorderError, RecordingEvent},
            peer_state::RecordingPeerState,
            state::RecordingState,
            StreamStatus, StreamTargetSecret, NAMESPACE, RECORD_FEATURE, STREAM_FEATURE,
        },
        Role,
    },
};
use snafu::{Report, ResultExt};
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

pub struct Recording {
    id: ParticipantId,
    room: SignalingRoomId,
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
            .module_features(NAMESPACE)
            .into_iter()
            .flatten()
            .filter_map(|feature| match feature.as_str() {
                RECORD_FEATURE => Some(RecordingFeature::Record),
                STREAM_FEATURE => Some(RecordingFeature::Stream),
                _ => None,
            })
            .collect();

        Ok(Some(Self {
            id: ctx.participant_id(),
            room: ctx.room_id(),
            params: params.clone(),
            enabled_features,
            db: ctx.db().clone(),
            rabbitmq_channel,
            recorder_started: false,
        }))
    }

    fn get_provided_features() -> Vec<&'static str> {
        vec![RECORD_FEATURE, STREAM_FEATURE]
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
                ctx.redis_conn()
                    .remove_attribute(self.room, self.id, "recording_consent")
                    .await?;
            }
            Event::RaiseHand => {}
            Event::LowerHand => {}
            Event::ParticipantLeft(_) => {}
            Event::ParticipantJoined(id, data) | Event::ParticipantUpdated(id, data) => {
                let consent: Option<bool> = ctx
                    .redis_conn()
                    .get_attribute(self.room, id, "recording_consent")
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
                RecordingCommand::SetConsent(command::SetConsent { consent }) => {
                    ctx.redis_conn()
                        .set_attribute(self.room, self.id, "recording_consent", consent)
                        .await?;

                    ctx.invalidate_data();
                }
                RecordingCommand::StartStream(command::StartStreaming { target_ids }) => {
                    self.handle_start_streams(&mut ctx, target_ids).await?
                }
                RecordingCommand::PauseStream(command::PauseStreaming { target_ids }) => {
                    self.handle_pause_streams(&mut ctx, target_ids).await?
                }
                RecordingCommand::StopStream(command::StopStreaming { target_ids }) => {
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

                    let streams = ctx.redis_conn().get_streams(self.room).await?;

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

                    ctx.redis_conn().set_streams(self.room, &streams).await?;

                    log::warn!("Recorder ran into a timeout!");
                    ctx.ws_send(RecorderError::Timeout);
                }
            },
        }

        Ok(())
    }

    async fn on_destroy(self, mut ctx: DestroyContext<'_>) {
        if ctx.destroy_room() {
            if let Err(e) = storage::delete_all_streams(ctx.redis_conn(), self.room).await {
                log::error!("failed to delete streams, {}", Report::from_error(e));
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

impl Recording {
    async fn initialize_streaming(
        &self,
        redis_conn: &mut RedisConnection,
    ) -> Result<(), SignalingModuleError> {
        let mut conn = self.db.get_conn().await?;

        let can_record = self.enabled_features.contains(&RecordingFeature::Record);
        let can_stream = self.enabled_features.contains(&RecordingFeature::Stream);

        let stock_streams = can_record.then_some((
            StreamingTargetId::generate(),
            StreamTargetSecret::recording(),
        ));
        let streams = if self.room.breakout_room_id().is_some() || !can_stream {
            BTreeMap::from_iter(stock_streams)
        } else {
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

        redis_conn.set_streams(self.room, &streams).await?;

        Ok(())
    }

    async fn handle_joined_event(
        &mut self,
        mut ctx: ModuleContext<'_, Self>,
        frontend_data: &mut Option<RecordingState>,
        participants: &mut HashMap<ParticipantId, Option<RecordingPeerState>>,
    ) -> Result<(), SignalingModuleError> {
        if !ctx.redis_conn().is_streaming_initialized(self.room).await? {
            self.initialize_streaming(ctx.redis_conn()).await?;
        }

        let streams_res = ctx.redis_conn().get_streams(self.room).await?;
        *frontend_data = Some({
            RecordingState {
                targets: BTreeMap::from_iter(
                    streams_res
                        .into_iter()
                        .map(|(target_id, stream_target)| (target_id, stream_target.into())),
                ),
            }
        });

        self.collect_participants_consents(ctx.redis_conn(), participants)
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

        if !self.target_ids_exist(ctx.redis_conn(), &target_ids).await? {
            ctx.ws_send(Error::InvalidStreamingId);
            return Ok(());
        }

        let is_recorder_running = ctx
            .redis_conn()
            .streams_contain_status(
                self.room,
                BTreeSet::from_iter([
                    StreamStatus::Active,
                    StreamStatus::Starting,
                    StreamStatus::Paused,
                ]),
            )
            .await?;

        ctx.redis_conn()
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
            RecordingService::NAMESPACE,
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

        if !self.target_ids_exist(ctx.redis_conn(), &target_ids).await? {
            ctx.ws_send(Error::InvalidStreamingId);
            return Ok(());
        }

        ctx.exchange_publish_to_namespace(
            control::exchange::current_room_all_recorders(self.room),
            RecordingService::NAMESPACE,
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

        if !self.target_ids_exist(ctx.redis_conn(), &target_ids).await? {
            ctx.ws_send(Error::InvalidStreamingId);
            return Ok(());
        }

        let is_recorder_running = ctx
            .redis_conn()
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
            RecordingService::NAMESPACE,
            recording_service::exchange::Message::StopStreams { target_ids },
        );

        Ok(())
    }

    async fn collect_participants_consents(
        &self,
        redis_conn: &mut RedisConnection,
        participants: &mut HashMap<ParticipantId, Option<RecordingPeerState>>,
    ) -> Result<(), SignalingModuleError> {
        let participant_ids: Vec<ParticipantId> = participants.keys().copied().collect();
        let participant_consents: Vec<Option<bool>> = redis_conn
            .get_attribute_for_participants(self.room, "recording_consent", &participant_ids)
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
        redis_conn: &mut RedisConnection,
        target_ids: &BTreeSet<StreamingTargetId>,
    ) -> Result<bool, SignalingModuleError> {
        for target_id in target_ids {
            if !redis_conn.stream_exists(self.room, *target_id).await? {
                return Ok(false);
            }
        }
        Ok(true)
    }
}
