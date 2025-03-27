// SPDX-FileCopyrightText: OpenTalk GmbH <mail@opentalk.eu>
//
// SPDX-License-Identifier: EUPL-1.2

use std::collections::BTreeMap;

use opentalk_signaling_core::{
    control, DestroyContext, Event, InitContext, ModuleContext, Participant, SignalingModule,
    SignalingModuleError, SignalingModuleInitData, SignalingRoomId,
};
use opentalk_types_common::{modules::ModuleId, streaming::StreamingTargetId};
use opentalk_types_signaling_recording::{StreamStatus, StreamTargetSecret, StreamUpdated};
use opentalk_types_signaling_recording_service::{
    command::RecordingServiceCommand,
    event::RecordingServiceEvent,
    state::{RecorderStreamInfo, RecordingServiceState},
    MODULE_ID,
};

use super::recording::{self, Recording, RecordingStorageProvider as _};

pub(crate) mod exchange;

#[derive(Debug)]
pub struct RecordingService {
    room: SignalingRoomId,
    /// Whether or not the current participant is the recorder
    is_recorder: bool,
}

#[async_trait::async_trait(?Send)]
impl SignalingModule for RecordingService {
    const NAMESPACE: ModuleId = MODULE_ID;

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
        let is_recorder = matches!(ctx.participant(), Participant::Recorder);
        Ok(Some(Self {
            room: ctx.room_id(),
            is_recorder,
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
            } => {
                if self.is_recorder {
                    self.handle_joined(ctx, frontend_data).await?;
                }
            }

            Event::Leaving => {
                if self.is_recorder {
                    self.handle_leaving(ctx).await?;
                }
            }
            Event::WsMessage(msg) => match msg {
                RecordingServiceEvent::StreamUpdated(stream_updated) => {
                    if self.is_recorder {
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
        let mut stream = ctx
            .volatile
            .storage()
            .get_stream(self.room, stream_updated.target_id)
            .await?;

        stream.status = stream_updated.status.clone();
        ctx.volatile
            .storage()
            .set_stream(self.room, stream_updated.target_id, stream)
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
        // notify recording module that the recorder is leaving
        ctx.exchange_publish_to_namespace(
            control::exchange::current_room_all_recorders(self.room),
            Recording::NAMESPACE,
            recording::exchange::Message::RecorderStopping,
        );

        let targets = ctx.volatile.storage().get_streams(self.room).await?;
        let targets: BTreeMap<StreamingTargetId, StreamTargetSecret> = targets
            .into_iter()
            .filter(|(_, target)| target.status != StreamStatus::Inactive)
            .map(|(id, mut target)| {
                target.status = StreamStatus::Inactive;
                (id, target)
            })
            .collect();

        if targets.is_empty() {
            return Ok(());
        }

        ctx.volatile
            .storage()
            .set_streams(self.room, &targets)
            .await?;

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
        mut ctx: ModuleContext<'_, Self>,
        frontend_data: &mut Option<RecordingServiceState>,
    ) -> Result<(), SignalingModuleError> {
        // Signal recording module that the recorder is started
        ctx.exchange_publish_to_namespace(
            control::exchange::current_room_all_recorders(self.room),
            Recording::NAMESPACE,
            recording::exchange::Message::RecorderStarting,
        );

        let streams = ctx
            .volatile
            .storage()
            .get_streams(self.room)
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
