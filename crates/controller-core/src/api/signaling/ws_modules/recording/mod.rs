// SPDX-FileCopyrightText: OpenTalk GmbH <mail@opentalk.eu>
//
// SPDX-License-Identifier: EUPL-1.2

use std::{
    collections::{BTreeMap, BTreeSet, HashMap},
    sync::Arc,
};

use lapin_pool::{RabbitMqChannel, RabbitMqPool};
use opentalk_database::Db;
use opentalk_db_storage::streaming_targets::RoomStreamingTargetRecord;
use opentalk_signaling_core::{
    control, DestroyContext, Event, InitContext, ModuleContext, RedisConnection, SignalingModule,
    SignalingModuleError, SignalingModuleInitData, SignalingRoomId,
};
use opentalk_types::{
    core::{ParticipantId, StreamingTargetId},
    signaling::{
        recording::{
            command::{self, RecordingCommand},
            event::{Error, RecordingEvent},
            peer_state::RecordingPeerState,
            state::RecordingState,
            StreamStatus, StreamTargetSecret, NAMESPACE,
        },
        Role,
    },
};
use snafu::{Report, ResultExt};

use super::recording_service::{self, RecordingService};

pub(crate) mod exchange;
mod rabbitmq;
pub(crate) mod storage;

pub struct Recording {
    id: ParticipantId,
    room: SignalingRoomId,
    params: RecordingParams,
    /// Whether or not the current participant is the recorder
    db: Arc<Db>,

    /// RabbitMQ channel used to send the recording start command over
    rabbitmq_channel: RabbitMqChannel,
}

#[derive(Clone)]
pub struct RecordingParams {
    pub queue: String,
}

#[async_trait::async_trait(?Send)]
impl SignalingModule for Recording {
    const NAMESPACE: &'static str = NAMESPACE;

    type Params = (Arc<RabbitMqPool>, RecordingParams);

    type Incoming = RecordingCommand;
    type Outgoing = RecordingEvent;
    type ExchangeMessage = exchange::Message;

    type ExtEvent = ();

    type FrontendData = RecordingState;
    type PeerFrontendData = RecordingPeerState;

    async fn init(
        ctx: InitContext<'_, Self>,
        params: &Self::Params,
        _protocol: &'static str,
    ) -> Result<Option<Self>, SignalingModuleError> {
        let (rabbitmq_pool, params) = params;

        let rabbitmq_channel = rabbitmq_pool.create_channel().await?;

        Ok(Some(Self {
            id: ctx.participant_id(),
            room: ctx.room_id(),
            params: params.clone(),
            db: ctx.db().clone(),
            rabbitmq_channel,
        }))
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
                control::storage::remove_attribute(
                    ctx.redis_conn(),
                    self.room,
                    self.id,
                    "recording_consent",
                )
                .await?;
            }
            Event::RaiseHand => {}
            Event::LowerHand => {}
            Event::ParticipantLeft(_) => {}
            Event::ParticipantJoined(id, data) | Event::ParticipantUpdated(id, data) => {
                let consent: Option<bool> = control::storage::get_attribute(
                    ctx.redis_conn(),
                    self.room,
                    id,
                    "recording_consent",
                )
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
                    control::storage::set_attribute(
                        ctx.redis_conn(),
                        self.room,
                        self.id,
                        "recording_consent",
                        consent,
                    )
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
            },
            Event::Ext(_) => {}
        }

        Ok(())
    }

    async fn on_destroy(self, mut ctx: DestroyContext<'_>) {
        if ctx.destroy_room() {
            if let Err(e) = storage::delete_all_streams(ctx.redis_conn(), self.room).await {
                log::error!("failed to delete streams, {:?}", e);
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

        let recorder_stream = (
            StreamingTargetId::generate(),
            StreamTargetSecret::recording(),
        );
        let streams = if self.room.breakout_room_id().is_some() {
            BTreeMap::from([recorder_stream])
        } else {
            let streaming_targets =
                RoomStreamingTargetRecord::get_all_for_room(&mut conn, self.room.room_id()).await?;
            std::iter::once(Ok(recorder_stream))
                .chain(streaming_targets.into_iter().map(|target| {
                    let id = target.id;
                    StreamTargetSecret::try_from(target)
                        .map(|stream_target_secret| (id, stream_target_secret))
                        .with_whatever_context::<_, _, SignalingModuleError>(|err| format!("{err}"))
                }))
                .collect::<Result<_, SignalingModuleError>>()?
        };

        storage::set_streams(redis_conn, self.room, &streams).await?;

        Ok(())
    }

    async fn handle_joined_event(
        &mut self,
        mut ctx: ModuleContext<'_, Self>,
        frontend_data: &mut Option<RecordingState>,
        participants: &mut HashMap<ParticipantId, Option<RecordingPeerState>>,
    ) -> Result<(), SignalingModuleError> {
        if !storage::is_streaming_initialized(ctx.redis_conn(), self.room).await? {
            self.initialize_streaming(ctx.redis_conn()).await?;
        }

        let streams_res = storage::get_streams(ctx.redis_conn(), self.room).await?;
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

        let is_recorder_running = storage::streams_contains_status(
            ctx.redis_conn(),
            self.room,
            vec![
                StreamStatus::Active,
                StreamStatus::Starting,
                StreamStatus::Paused,
            ],
        )
        .await?;

        storage::update_streams(
            ctx.redis_conn(),
            self.room,
            &target_ids,
            StreamStatus::Starting,
        )
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

        let is_recorder_running = storage::streams_contains_status(
            ctx.redis_conn(),
            self.room,
            vec![
                StreamStatus::Active,
                StreamStatus::Starting,
                StreamStatus::Paused,
            ],
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
        let participant_consents: Vec<Option<bool>> =
            control::storage::get_attribute_for_participants(
                redis_conn,
                self.room,
                "recording_consent",
                &participant_ids,
            )
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
            if !storage::stream_exists(redis_conn, self.room, *target_id).await? {
                return Ok(false);
            }
        }
        Ok(true)
    }
}
