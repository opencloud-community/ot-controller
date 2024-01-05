// SPDX-FileCopyrightText: OpenTalk GmbH <mail@opentalk.eu>
//
// SPDX-License-Identifier: EUPL-1.2

//! Breakout room module

use self::storage::BreakoutConfig;
use actix_http::ws::CloseCode;
use anyhow::{bail, Result};
use futures::FutureExt;
use signaling_core::{
    control, DestroyContext, Event, InitContext, ModuleContext, SignalingModule,
    SignalingModuleInitData, SignalingRoomId,
};
use std::collections::HashMap;
use std::time::{Duration, SystemTime};
use tokio::time::sleep;
use types::signaling::breakout::NAMESPACE;
use types::{
    core::{BreakoutRoomId, ParticipantId, RoomId},
    signaling::{
        breakout::{
            command::BreakoutCommand,
            event::{BreakoutEvent, Error, Started},
            state::BreakoutState,
            AssociatedParticipantInOtherRoom, BreakoutRoom, ParticipantInOtherRoom,
        },
        Role,
    },
};

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
    ) -> Result<Option<Self>> {
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
    ) -> Result<()> {
        match event {
            Event::Joined {
                control_data,
                frontend_data,
                participants: _,
            } => {
                if let Some(config) = storage::get_config(ctx.redis_conn(), self.parent).await? {
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
                            bail!("joined already expired room")
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
                    bail!("Inside breakout room, but no config is set");
                }

                Ok(())
            }
            Event::Leaving => {
                let config = storage::get_config(ctx.redis_conn(), self.parent).await?;

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

    async fn build_params(_init: SignalingModuleInitData) -> Result<Option<Self::Params>> {
        Ok(Some(()))
    }
}

impl BreakoutRooms {
    async fn add_room_to_participants_list(
        &mut self,
        ctx: &mut ModuleContext<'_, Self>,
        room: SignalingRoomId,
        list: &mut Vec<ParticipantInOtherRoom>,
    ) -> Result<()> {
        let breakout_room_participants =
            control::storage::get_all_participants(ctx.redis_conn(), room).await?;

        for participant in breakout_room_participants {
            let res = control::storage::AttrPipeline::new(room, participant)
                .get("display_name")
                .get("role")
                .get("avatar_url")
                .get("kind")
                .get("joined_at")
                .get("left_at")
                .query_async(ctx.redis_conn())
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
    ) -> Result<()> {
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

                storage::set_config(ctx.redis_conn(), self.parent, &config).await?;

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
                if storage::del_config(ctx.redis_conn(), self.parent).await? {
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
    ) -> Result<()> {
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
