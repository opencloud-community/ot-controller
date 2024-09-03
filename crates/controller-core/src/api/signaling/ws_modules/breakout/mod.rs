// SPDX-FileCopyrightText: OpenTalk GmbH <mail@opentalk.eu>
//
// SPDX-License-Identifier: EUPL-1.2

//! Breakout room module

use std::{
    collections::HashMap,
    time::{Duration, SystemTime},
};

use actix_http::ws::CloseCode;
use either::Either;
use futures::FutureExt;
use opentalk_signaling_core::{
    control::{
        self,
        storage::{
            AttributeActions, ControlStorageParticipantAttributes as _, AVATAR_URL, DISPLAY_NAME,
            JOINED_AT, KIND, LEFT_AT, ROLE,
        },
    },
    DestroyContext, Event, InitContext, ModuleContext, SignalingModule, SignalingModuleError,
    SignalingModuleInitData, SignalingRoomId, VolatileStorage,
};
use opentalk_types::signaling::breakout::{
    command::BreakoutCommand, event::BreakoutEvent, state::BreakoutState, NAMESPACE,
};
use opentalk_types_common::rooms::{BreakoutRoomId, RoomId};
use opentalk_types_signaling::{ParticipantId, Role};
use opentalk_types_signaling_breakout::{
    event::{Error, Started},
    AssociatedParticipantInOtherRoom, BreakoutRoom, ParticipantInOtherRoom,
};
use snafu::whatever;
use tokio::time::sleep;

use self::storage::{BreakoutConfig, BreakoutStorage};

pub mod exchange;
pub mod storage;

pub struct BreakoutRooms {
    id: ParticipantId,
    parent: RoomId,
    room: SignalingRoomId,
    breakout_room: Option<BreakoutRoomId>,
}

pub enum TimerEvent {
    RoomExpired,
    LeavePeriodExpired,
}

pub(crate) trait BreakoutStorageProvider {
    fn breakout_storage(&mut self) -> &mut dyn BreakoutStorage;
}

impl BreakoutStorageProvider for VolatileStorage {
    fn breakout_storage(&mut self) -> &mut dyn BreakoutStorage {
        match self.as_mut() {
            Either::Left(v) => v,
            Either::Right(v) => v,
        }
    }
}

#[async_trait::async_trait(?Send)]
impl SignalingModule for BreakoutRooms {
    const NAMESPACE: &'static str = NAMESPACE;

    type Params = ();
    type Incoming = BreakoutCommand;
    type Outgoing = BreakoutEvent;
    type ExchangeMessage = exchange::Message;
    type ExtEvent = TimerEvent;
    type FrontendData = BreakoutState;
    type PeerFrontendData = ();

    async fn init(
        ctx: InitContext<'_, Self>,
        _params: &Self::Params,
        _protocol: &'static str,
    ) -> Result<Option<Self>, SignalingModuleError> {
        Ok(Some(Self {
            id: ctx.participant_id(),
            parent: ctx.room().id,
            room: ctx.room_id(),
            breakout_room: ctx.breakout_room(),
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
                frontend_data,
                participants: _,
            } => {
                if let Some(config) = ctx
                    .volatile
                    .breakout_storage()
                    .get_breakout_config(self.parent)
                    .await?
                {
                    let mut expires = None;

                    // If the breakout rooms have an expiry, calculate a rough `Duration` to sleep.
                    if let Some(duration) = config.duration {
                        // Get the time the room is running. In case of a future timestamp, just assume elapsed = 0 (default)
                        let elapsed = config.started.elapsed().unwrap_or_default();

                        // Checked sub in case elapsed > duration which means the breakout room is already expired
                        let remaining = duration.checked_sub(elapsed);

                        if let Some(remaining) = remaining {
                            // Create room expiry event
                            ctx.add_event_stream(futures::stream::once(
                                sleep(remaining).map(|_| TimerEvent::RoomExpired),
                            ));

                            expires = Some((config.started + duration).into())
                        } else if self.breakout_room.is_some() {
                            // The breakout room is expired and we tried to join it, exit here
                            ctx.exit(None);
                            whatever!("joined already expired room")
                        }
                    }

                    ctx.exchange_publish(
                        control::exchange::global_room_all_participants(self.room.room_id()),
                        exchange::Message::Joined(ParticipantInOtherRoom {
                            breakout_room: self.breakout_room,
                            id: self.id,
                            display_name: control_data.display_name.clone(),
                            role: control_data.role,
                            avatar_url: control_data.avatar_url.clone(),
                            participation_kind: control_data.participation_kind,
                            joined_at: control_data.joined_at,
                            left_at: None,
                        }),
                    );

                    // Collect all participants inside all other breakout rooms
                    let mut participants = Vec::new();

                    // When inside a breakout room collect participants from the parent room
                    if self.breakout_room.is_some() {
                        let parent_room_id = SignalingRoomId::new_for_room(self.room.room_id());

                        self.add_room_to_participants_list(
                            &mut ctx,
                            parent_room_id,
                            &mut participants,
                        )
                        .await?;
                    }

                    // Collect all participant from all other breakout rooms, expect the one we were inside
                    for breakout_room in &config.rooms {
                        // skip own room if inside one
                        if self
                            .breakout_room
                            .map(|self_breakout_room| self_breakout_room == breakout_room.id)
                            .unwrap_or_default()
                        {
                            continue;
                        }

                        // get the full room id of the breakout room to access the storage and such
                        let full_room_id =
                            SignalingRoomId::new(self.room.room_id(), Some(breakout_room.id));

                        self.add_room_to_participants_list(
                            &mut ctx,
                            full_room_id,
                            &mut participants,
                        )
                        .await?;
                    }

                    *frontend_data = Some(BreakoutState {
                        current: self.breakout_room,
                        expires,
                        rooms: config.rooms,
                        participants,
                    });
                } else if self.breakout_room.is_some() {
                    ctx.exit(Some(CloseCode::Error));
                    whatever!("Inside breakout room, but no config is set")
                }

                Ok(())
            }
            Event::Leaving => {
                let config = ctx
                    .volatile
                    .breakout_storage()
                    .get_breakout_config(self.parent)
                    .await?;

                if config.is_some() || self.breakout_room.is_some() {
                    ctx.exchange_publish(
                        control::exchange::global_room_all_participants(self.room.room_id()),
                        exchange::Message::Left(AssociatedParticipantInOtherRoom {
                            breakout_room: self.breakout_room,
                            id: self.id,
                        }),
                    );
                }

                Ok(())
            }
            Event::RaiseHand => Ok(()),
            Event::LowerHand => Ok(()),
            Event::ParticipantJoined(_, _) => Ok(()),
            Event::ParticipantLeft(_) => Ok(()),
            Event::ParticipantUpdated(_, _) => Ok(()),
            Event::RoleUpdated(_) => Ok(()),
            Event::WsMessage(msg) => self.on_ws_msg(ctx, msg).await,
            Event::Exchange(msg) => self.on_exchange_msg(ctx, msg).await,
            Event::Ext(TimerEvent::RoomExpired) => {
                ctx.ws_send(BreakoutEvent::Expired);

                if self.breakout_room.is_some() {
                    // Create timer to force leave the room after 5min
                    ctx.add_event_stream(futures::stream::once(
                        sleep(Duration::from_secs(60 * 5)).map(|_| TimerEvent::LeavePeriodExpired),
                    ));
                }

                Ok(())
            }
            Event::Ext(TimerEvent::LeavePeriodExpired) => {
                if self.breakout_room.is_some() {
                    // The 5min leave period expired, force quit
                    ctx.exit(None);
                }

                Ok(())
            }
        }
    }

    async fn on_destroy(self, _ctx: DestroyContext<'_>) {}

    async fn build_params(
        _init: SignalingModuleInitData,
    ) -> Result<Option<Self::Params>, SignalingModuleError> {
        Ok(Some(()))
    }
}

impl BreakoutRooms {
    async fn add_room_to_participants_list(
        &mut self,
        ctx: &mut ModuleContext<'_, Self>,
        room: SignalingRoomId,
        list: &mut Vec<ParticipantInOtherRoom>,
    ) -> Result<(), SignalingModuleError> {
        let breakout_room_participants = ctx
            .volatile
            .breakout_storage()
            .get_all_participants(room)
            .await?;

        for participant in breakout_room_participants {
            let res = ctx
                .volatile
                .breakout_storage()
                .bulk_attribute_actions(
                    AttributeActions::new(room, participant)
                        .get(DISPLAY_NAME)
                        .get(ROLE)
                        .get(AVATAR_URL)
                        .get(KIND)
                        .get(JOINED_AT)
                        .get(LEFT_AT),
                )
                .await;

            match res {
                Ok((display_name, role, avatar_url, kind, joined_at, left_at)) => {
                    list.push(ParticipantInOtherRoom {
                        breakout_room: room.breakout_room_id(),
                        id: participant,
                        display_name,
                        role,
                        avatar_url,
                        participation_kind: kind,
                        joined_at,
                        left_at,
                    })
                }
                Err(e) => log::error!(
                    "Failed to fetch participant data from {} in {}, {:?}",
                    participant,
                    room,
                    e
                ),
            }
        }

        Ok(())
    }

    async fn on_ws_msg(
        &mut self,
        mut ctx: ModuleContext<'_, Self>,
        msg: BreakoutCommand,
    ) -> Result<(), SignalingModuleError> {
        if ctx.role() != Role::Moderator {
            ctx.ws_send(Error::InsufficientPermissions);
            return Ok(());
        }

        match msg {
            BreakoutCommand::Start(start) => {
                if start.rooms.is_empty() {
                    // Discard message, case should be handled by frontend
                    return Ok(());
                }

                let started = SystemTime::now();

                let mut rooms = vec![];
                let mut assignments = HashMap::new();

                for room_param in start.rooms {
                    let id = BreakoutRoomId::generate();

                    for assigned_participant_id in room_param.assignments {
                        assignments.insert(assigned_participant_id, id);
                    }

                    rooms.push(BreakoutRoom {
                        id,
                        name: room_param.name,
                    });
                }

                let config = BreakoutConfig {
                    rooms,
                    started,
                    duration: start.duration,
                };

                ctx.volatile
                    .breakout_storage()
                    .set_breakout_config(self.parent, &config)
                    .await?;

                ctx.exchange_publish(
                    control::exchange::global_room_all_participants(self.parent),
                    exchange::Message::Start(exchange::Start {
                        config,
                        started,
                        assignments,
                    }),
                );
            }
            BreakoutCommand::Stop => {
                if ctx
                    .volatile
                    .breakout_storage()
                    .del_breakout_config(self.parent)
                    .await?
                {
                    ctx.exchange_publish(
                        control::exchange::global_room_all_participants(self.parent),
                        exchange::Message::Stop,
                    );
                } else {
                    ctx.ws_send(Error::Inactive);
                }
            }
        }

        Ok(())
    }

    async fn on_exchange_msg(
        &mut self,
        mut ctx: ModuleContext<'_, Self>,
        msg: exchange::Message,
    ) -> Result<(), SignalingModuleError> {
        match msg {
            exchange::Message::Start(start) => {
                let assignment = start.assignments.get(&self.id).copied();

                let expires = if let Some(duration) = start.config.duration {
                    Some((start.started + duration).into())
                } else {
                    None
                };

                ctx.ws_send(Started {
                    rooms: start.config.rooms,
                    expires,
                    assignment,
                });
            }
            exchange::Message::Stop => {
                ctx.ws_send(BreakoutEvent::Stopped);

                if self.breakout_room.is_some() {
                    // Create timer to force leave the room after 5min
                    ctx.add_event_stream(futures::stream::once(
                        sleep(Duration::from_secs(60 * 5)).map(|_| TimerEvent::LeavePeriodExpired),
                    ));
                }
            }
            exchange::Message::Joined(participant) => {
                if self.breakout_room == participant.breakout_room {
                    return Ok(());
                }

                ctx.ws_send(BreakoutEvent::Joined(participant))
            }
            exchange::Message::Left(assoc_participant) => {
                if self.breakout_room == assoc_participant.breakout_room {
                    return Ok(());
                }

                ctx.ws_send(BreakoutEvent::Left(assoc_participant))
            }
        }

        Ok(())
    }
}
