// SPDX-FileCopyrightText: OpenTalk GmbH <mail@opentalk.eu>
//
// SPDX-License-Identifier: EUPL-1.2

use std::collections::BTreeMap;

use opentalk_signaling_core::{
    control, DestroyContext, Event, InitContext, ModuleContext, Participant, RedisConnection,
    SignalingModule, SignalingModuleError, SignalingModuleInitData, SignalingRoomId,
};
use opentalk_types::{
    core::StreamingTargetId,
    signaling::{
        recording::{state::RecorderStreamInfo, StreamStatus, StreamTargetSecret, StreamUpdated},
        recording_service::{
            command::RecordingServiceCommand, event::RecordingServiceEvent,
            state::RecordingServiceState, NAMESPACE,
        },
    },
};

use super::recording::{self, Recording};

pub(crate) mod exchange;

#[derive(Debug)]
pub struct RecordingService {
    room: SignalingRoomId,
    /// Whether or not the current participant is the recorder
    i_am_the_recorder: bool,
}

#[async_trait::async_trait(?Send)]
impl SignalingModule for RecordingService {
    const NAMESPACE: &'static str = NAMESPACE;

    type Params = ();

    type Incoming = RecordingServiceEvent;
    type Outgoing = RecordingServiceCommand;

    type ExchangeMessage = exchange::Message;

    type ExtEvent = ();

    type FrontendData = RecordingServiceState;
    type PeerFrontendData = ();

    async fn init(
        ctx: InitContext<'_, Self>,
        _params: &Self::Params,
        _protocol: &'static str,
    ) -> Result<Option<Self>, SignalingModuleError> {
        let i_am_the_recorder = matches!(ctx.participant(), Participant::Recorder);
        Ok(Some(Self {
            room: ctx.room_id(),
            i_am_the_recorder,
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
                participants: _,
            } => self.handle_joined(ctx.redis_conn(), frontend_data).await?,

            Event::Leaving => {
                if self.i_am_the_recorder {
                    self.handle_leaving(ctx).await?;
                }
            }
            Event::WsMessage(msg) => match msg {
                RecordingServiceEvent::StreamUpdated(stream_updated) => {
                    if self.i_am_the_recorder {
                        self.handle_stream_updated(&mut ctx, stream_updated).await?;
                    }
                }
            },
            // Messages from other controllers (Exchange)
            Event::Exchange(msg) => match msg {
                exchange::Message::StartStreams { target_ids } => {
                    ctx.ws_send(RecordingServiceCommand::StartStreams { target_ids });
                }
                exchange::Message::PauseStreams { target_ids } => {
                    ctx.ws_send(RecordingServiceCommand::PauseStreams { target_ids });
                }
                exchange::Message::StopStreams { target_ids } => {
                    ctx.ws_send(RecordingServiceCommand::StopStreams { target_ids });
                }
            },
            _ => return Ok(()),
        }

        Ok(())
    }

    async fn on_destroy(self, mut _ctx: DestroyContext<'_>) {}

    async fn build_params(
        _init: SignalingModuleInitData,
    ) -> Result<Option<Self::Params>, SignalingModuleError> {
        Ok(Some(()))
    }
}

impl RecordingService {
    pub async fn handle_stream_updated(
        &self,
        ctx: &mut ModuleContext<'_, Self>,
        stream_updated: StreamUpdated,
    ) -> Result<(), SignalingModuleError> {
        let mut stream =
            recording::storage::get_stream(ctx.redis_conn(), self.room, stream_updated.target_id)
                .await?;

        stream.status = stream_updated.status.clone();
        recording::storage::set_stream(
            ctx.redis_conn(),
            self.room,
            stream_updated.target_id,
            stream,
        )
        .await?;

        ctx.exchange_publish_to_namespace(
            control::exchange::current_room_all_participants(self.room),
            Recording::NAMESPACE,
            recording::exchange::Message::StreamUpdated(stream_updated),
        );
        Ok(())
    }

    pub async fn handle_leaving(
        &self,
        mut ctx: ModuleContext<'_, Self>,
    ) -> Result<(), SignalingModuleError> {
        let targets = recording::storage::get_streams(ctx.redis_conn(), self.room).await?;
        let targets = targets
            .into_iter()
            .filter(|(_, target)| target.status != StreamStatus::Active)
            .collect();
        recording::storage::set_streams(ctx.redis_conn(), self.room, &targets).await?;

        for stream_updated in targets.iter().map(|(target_id, target)| StreamUpdated {
            target_id: *target_id,
            status: target.status.clone(),
        }) {
            ctx.exchange_publish_to_namespace(
                control::exchange::current_room_all_participants(self.room),
                Recording::NAMESPACE,
                recording::exchange::Message::StreamUpdated(stream_updated),
            );
        }

        Ok(())
    }

    pub async fn handle_joined(
        &self,
        redis_conn: &mut RedisConnection,
        frontend_data: &mut Option<RecordingServiceState>,
    ) -> Result<(), SignalingModuleError> {
        let streams = recording::storage::get_streams(redis_conn, self.room)
            .await?
            .into_iter()
            .map(|(id, target)| {
                (
                    id,
                    StreamTargetSecret {
                        name: target.name,
                        kind: target.kind,
                        status: target.status,
                    },
                )
            })
            .collect::<BTreeMap<StreamingTargetId, StreamTargetSecret>>();

        let streams = streams
            .into_iter()
            .map(|(id, target)| (id, target.into()))
            .collect::<BTreeMap<StreamingTargetId, RecorderStreamInfo>>();

        *frontend_data = Some(RecordingServiceState { streams });

        Ok(())
    }
}
