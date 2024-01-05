// SPDX-FileCopyrightText: OpenTalk GmbH <mail@opentalk.eu>
//
// SPDX-License-Identifier: EUPL-1.2

use anyhow::{Context, Result};
use lapin_pool::{RabbitMqChannel, RabbitMqPool};
use signaling_core::{
    control, DestroyContext, Event, InitContext, ModuleContext, Participant, SignalingModule,
    SignalingModuleInitData, SignalingRoomId,
};
use std::sync::Arc;
use types::{
    core::ParticipantId,
    signaling::{
        recording::{
            command::{self, RecordingCommand},
            event::{Error, RecordingEvent, Started, Stopped},
            peer_state::RecordingPeerState,
            state::RecordingState,
            RecordingId, RecordingStatus, NAMESPACE,
        },
        Role,
    },
};

mod exchange;
mod rabbitmq;
mod storage;

pub struct Recording {
    id: ParticipantId,
    room: SignalingRoomId,
    i_am_the_recorder: bool,
    params: RecordingParams,

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
    ) -> Result<Option<Self>> {
        let (rabbitmq_pool, params) = params;

        let rabbitmq_channel = rabbitmq_pool.create_channel().await?;

        Ok(Some(Self {
            id: ctx.participant_id(),
            room: ctx.room_id(),
            i_am_the_recorder: matches!(ctx.participant(), Participant::Recorder),
            params: params.clone(),
            rabbitmq_channel,
        }))
    }

    async fn on_event(
        &mut self,
        mut ctx: ModuleContext<'_, Self>,
        event: Event<'_, Self>,
    ) -> Result<()> {
        match event {
            Event::Joined {
                control_data: _,
                frontend_data,
                participants,
            } => {
                if self.i_am_the_recorder {
                    let recording_id = RecordingId::from(self.id);
                    storage::set_recording(ctx.redis_conn(), self.room, recording_id).await?;

                    ctx.exchange_publish(
                        control::exchange::current_room_all_participants(self.room),
                        exchange::Message::Started(recording_id),
                    );
                } else {
                    *frontend_data = Some(RecordingState(
                        storage::get_state(ctx.redis_conn(), self.room).await?,
                    ));
                }

                let participant_ids: Vec<ParticipantId> = participants.keys().copied().collect();

                let participant_consents: Vec<Option<bool>> =
                    control::storage::get_attribute_for_participants(
                        ctx.redis_conn(),
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
            }
            Event::Leaving => {
                if self.i_am_the_recorder {
                    ctx.exchange_publish(
                        control::exchange::current_room_all_participants(self.room),
                        exchange::Message::Stopped(RecordingId::from(self.id)),
                    );
                } else {
                    control::storage::remove_attribute(
                        ctx.redis_conn(),
                        self.room,
                        self.id,
                        "recording_consent",
                    )
                    .await?;
                }
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
            Event::WsMessage(msg) => match msg {
                RecordingCommand::Start => {
                    if ctx.role() != Role::Moderator {
                        ctx.ws_send(Error::InsufficientPermissions);
                        return Ok(());
                    }

                    if !storage::try_init(ctx.redis_conn(), self.room).await? {
                        ctx.ws_send(Error::AlreadyRecording);
                        return Ok(());
                    }

                    self.rabbitmq_channel
                        .basic_publish(
                            "",
                            &self.params.queue,
                            Default::default(),
                            &serde_json::to_vec(&rabbitmq::StartRecording {
                                room: self.room.room_id(),
                                breakout: self.room.breakout_room_id(),
                            })
                            .context("failed to serialize StartRecording")?,
                            Default::default(),
                        )
                        .await?;
                }
                RecordingCommand::Stop(command::Stop { recording_id }) => {
                    if ctx.role() != Role::Moderator {
                        ctx.ws_send(Error::InsufficientPermissions);
                        return Ok(());
                    }

                    if !matches!(
                        storage::get_state(ctx.redis_conn(), self.room).await?,
                        Some(RecordingStatus::Recording(id)) if id == recording_id
                    ) {
                        ctx.ws_send(Error::InvalidRecordingId);
                        return Ok(());
                    }

                    ctx.exchange_publish(
                        control::exchange::current_room_by_participant_id(
                            self.room,
                            recording_id.into(),
                        ),
                        exchange::Message::Stop,
                    );
                }
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
            },
            Event::Exchange(msg) => match msg {
                exchange::Message::Stop => {
                    if self.i_am_the_recorder {
                        // TODO(kbalt): A bit of a nuclear solution to end the recording
                        ctx.exit(None);
                    }
                }
                exchange::Message::Started(recording_id) => {
                    if !self.i_am_the_recorder {
                        ctx.ws_send(Started { recording_id });
                    }
                }
                exchange::Message::Stopped(recording_id) => {
                    if !self.i_am_the_recorder {
                        control::storage::remove_attribute(
                            ctx.redis_conn(),
                            self.room,
                            self.id,
                            "recording_consent",
                        )
                        .await?;

                        ctx.ws_send(Stopped { recording_id });
                    }
                }
            },
            Event::Ext(_) => {}
        }

        Ok(())
    }

    async fn on_destroy(self, mut ctx: DestroyContext<'_>) {
        if self.i_am_the_recorder {
            if let Err(e) = storage::del_state(ctx.redis_conn(), self.room).await {
                log::error!("failed to delete state, {:?}", e);
            }
        }
    }

    async fn build_params(init: SignalingModuleInitData) -> Result<Option<Self::Params>> {
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
