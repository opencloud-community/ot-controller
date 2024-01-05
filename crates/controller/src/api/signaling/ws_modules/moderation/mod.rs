// SPDX-FileCopyrightText: OpenTalk GmbH <mail@opentalk.eu>
//
// SPDX-License-Identifier: EUPL-1.2

use actix_http::ws::CloseCode;
use anyhow::Result;
use signaling_core::{
    control::{self, ControlStateExt as _},
    DestroyContext, Event, InitContext, ModuleContext, RedisConnection, SignalingModule,
    SignalingModuleInitData, SignalingRoomId,
};
use std::iter::zip;
use types::{
    core::{ParticipantId, RoomId, UserId},
    signaling::{
        control::{state::ControlState, AssociatedParticipant, Participant, WaitingRoomState},
        moderation::{
            command::ModerationCommand,
            event::{Error, ModerationEvent},
            state::{ModerationState, ModeratorFrontendData},
        },
        ModulePeerData, Role,
    },
};

pub use types::signaling::moderation::NAMESPACE;

use crate::api::signaling::ws::ModuleContextExt;

pub mod exchange;
pub mod storage;

pub struct ModerationModule {
    room: SignalingRoomId,
    id: ParticipantId,
}

async fn build_waiting_room_participants(
    redis_conn: &mut RedisConnection,
    room_id: RoomId,
    list: &Vec<ParticipantId>,
    waiting_room_state: WaitingRoomState,
) -> Result<Vec<Participant>> {
    let mut waiting_room = Vec::with_capacity(list.len());

    for id in list {
        let control_data =
            ControlState::from_redis(redis_conn, SignalingRoomId::new(room_id, None), *id).await?;

        let mut module_data = ModulePeerData::new();
        module_data.insert(&control_data)?;
        module_data.insert(&waiting_room_state)?;

        waiting_room.push(Participant {
            id: *id,
            module_data,
        });
    }

    Ok(waiting_room)
}

async fn set_waiting_room_enabled(
    ctx: &mut ModuleContext<'_, ModerationModule>,
    room_id: RoomId,
    enabled: bool,
) -> Result<()> {
    storage::set_waiting_room_enabled(ctx.redis_conn(), room_id, enabled).await?;

    ctx.exchange_publish(
        control::exchange::global_room_all_participants(room_id),
        exchange::Message::WaitingRoomEnableUpdated,
    );

    Ok(())
}

#[async_trait::async_trait(?Send)]
impl SignalingModule for ModerationModule {
    const NAMESPACE: &'static str = NAMESPACE;

    type Params = ();
    type Incoming = ModerationCommand;
    type Outgoing = ModerationEvent;
    type ExchangeMessage = exchange::Message;
    type ExtEvent = ();
    type FrontendData = ModerationState;
    type PeerFrontendData = ();

    async fn init(
        ctx: InitContext<'_, Self>,
        _params: &Self::Params,
        _protocol: &'static str,
    ) -> Result<Option<Self>> {
        Ok(Some(Self {
            room: ctx.room_id(),
            id: ctx.participant_id(),
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
                participants: _,
            } => {
                let moderator_data = if ctx.role() == Role::Moderator {
                    let waiting_room_enabled =
                        storage::is_waiting_room_enabled(ctx.redis_conn(), self.room.room_id())
                            .await?;

                    let list =
                        storage::waiting_room_all(ctx.redis_conn(), self.room.room_id()).await?;
                    let mut waiting_room_participants = build_waiting_room_participants(
                        ctx.redis_conn(),
                        self.room.room_id(),
                        &list,
                        WaitingRoomState::Waiting,
                    )
                    .await?;

                    let list =
                        storage::waiting_room_accepted_all(ctx.redis_conn(), self.room.room_id())
                            .await?;
                    let mut accepted_waiting_room_participants = build_waiting_room_participants(
                        ctx.redis_conn(),
                        self.room.room_id(),
                        &list,
                        WaitingRoomState::Accepted,
                    )
                    .await?;

                    waiting_room_participants.append(&mut accepted_waiting_room_participants);

                    Some(ModeratorFrontendData {
                        waiting_room_enabled,
                        waiting_room_participants,
                    })
                } else {
                    None
                };

                let raise_hands_enabled =
                    storage::is_raise_hands_enabled(ctx.redis_conn(), self.room.room_id()).await?;

                *frontend_data = Some(ModerationState {
                    moderator_data,
                    raise_hands_enabled,
                });
            }
            Event::Leaving => {}
            Event::RaiseHand => {}
            Event::LowerHand => {}
            Event::ParticipantJoined(_, _) => {}
            Event::ParticipantLeft(_) => {}
            Event::ParticipantUpdated(_, _) => {}
            Event::RoleUpdated(_) => {}
            Event::WsMessage(ModerationCommand::Ban { target }) => {
                if ctx.role() != Role::Moderator {
                    return Ok(());
                }

                storage::waiting_room_accepted_remove(
                    ctx.redis_conn(),
                    self.room.room_id(),
                    target,
                )
                .await?;

                let user_id: Option<UserId> =
                    control::storage::get_attribute(ctx.redis_conn(), self.room, target, "user_id")
                        .await?;

                if let Some(user_id) = user_id {
                    storage::ban_user(ctx.redis_conn(), self.room.room_id(), user_id).await?;
                } else {
                    ctx.ws_send(Error::CannotBanGuest);
                    return Ok(());
                }

                ctx.exchange_publish(
                    control::exchange::current_room_by_participant_id(self.room, target),
                    exchange::Message::Banned(target),
                );
            }
            Event::WsMessage(ModerationCommand::Kick { target }) => {
                if ctx.role() != Role::Moderator {
                    return Ok(());
                }

                // Enforce the participant to enter the waiting room (if enabled) on next rejoin
                control::storage::set_skip_waiting_room_with_expiry(
                    ctx.redis_conn(),
                    target,
                    false,
                )
                .await?;

                storage::waiting_room_accepted_remove(
                    ctx.redis_conn(),
                    self.room.room_id(),
                    target,
                )
                .await?;

                ctx.exchange_publish(
                    control::exchange::current_room_by_participant_id(self.room, target),
                    exchange::Message::Kicked(target),
                );
            }
            Event::WsMessage(ModerationCommand::Debrief(kick_scope)) => {
                if ctx.role() != Role::Moderator {
                    return Ok(());
                }

                // Remove all debriefed participants from the waiting-room-accepted set
                let all_participants =
                    control::storage::get_all_participants(ctx.redis_conn(), self.room).await?;
                let all_participants_role: Vec<Option<Role>> =
                    control::storage::get_attribute_for_participants(
                        ctx.redis_conn(),
                        self.room,
                        "role",
                        &all_participants,
                    )
                    .await?;

                let mut to_remove = vec![];

                for (id, role) in zip(all_participants, all_participants_role) {
                    let remove = if let Some(role) = role {
                        kick_scope.kicks_role(role)
                    } else {
                        true
                    };

                    if remove {
                        // Enforce the participant to enter the waiting room on next rejoin
                        control::storage::set_skip_waiting_room_with_expiry(
                            ctx.redis_conn(),
                            id,
                            false,
                        )
                        .await?;

                        to_remove.push(id);
                    }
                }

                storage::waiting_room_accepted_remove_list(
                    ctx.redis_conn(),
                    self.room.room_id(),
                    &to_remove,
                )
                .await?;

                set_waiting_room_enabled(&mut ctx, self.room.room_id(), true).await?;

                ctx.exchange_publish(
                    control::exchange::current_room_all_participants(self.room),
                    exchange::Message::Debriefed {
                        kick_scope,
                        issued_by: self.id,
                    },
                );
            }
            Event::WsMessage(ModerationCommand::EnableWaitingRoom) => {
                if ctx.role() != Role::Moderator {
                    return Ok(());
                }

                set_waiting_room_enabled(&mut ctx, self.room.room_id(), true).await?;

                ctx.exchange_publish(
                    control::exchange::global_room_all_participants(self.room.room_id()),
                    exchange::Message::WaitingRoomEnableUpdated,
                );
            }
            Event::WsMessage(ModerationCommand::DisableWaitingRoom) => {
                if ctx.role() != Role::Moderator {
                    return Ok(());
                }

                set_waiting_room_enabled(&mut ctx, self.room.room_id(), false).await?;

                ctx.exchange_publish(
                    control::exchange::global_room_all_participants(self.room.room_id()),
                    exchange::Message::WaitingRoomEnableUpdated,
                );
            }
            Event::WsMessage(ModerationCommand::Accept { target }) => {
                if ctx.role() != Role::Moderator {
                    return Ok(());
                }

                if !storage::waiting_room_contains(ctx.redis_conn(), self.room.room_id(), target)
                    .await?
                {
                    // TODO return error
                    return Ok(());
                }

                storage::waiting_room_accepted_add(ctx.redis_conn(), self.room.room_id(), target)
                    .await?;
                storage::waiting_room_remove(ctx.redis_conn(), self.room.room_id(), target).await?;

                ctx.exchange_publish_control(
                    control::exchange::global_room_by_participant_id(self.room.room_id(), target),
                    control::exchange::Message::Accepted(target),
                );
            }
            Event::WsMessage(ModerationCommand::ResetRaisedHands) => {
                if ctx.role() != Role::Moderator {
                    return Ok(());
                }

                ctx.exchange_publish_control(
                    control::exchange::current_room_all_participants(self.room),
                    control::exchange::Message::ResetRaisedHands { issued_by: self.id },
                );
            }

            Event::WsMessage(ModerationCommand::EnableRaiseHands) => {
                if ctx.role() != Role::Moderator {
                    return Ok(());
                }

                storage::set_raise_hands_enabled(ctx.redis_conn(), self.room.room_id(), true)
                    .await?;

                ctx.exchange_publish_control(
                    control::exchange::current_room_all_participants(self.room),
                    control::exchange::Message::EnableRaiseHands { issued_by: self.id },
                );
            }

            Event::WsMessage(ModerationCommand::DisableRaiseHands) => {
                if ctx.role() != Role::Moderator {
                    return Ok(());
                }

                storage::set_raise_hands_enabled(ctx.redis_conn(), self.room.room_id(), false)
                    .await?;

                ctx.exchange_publish_control(
                    control::exchange::current_room_all_participants(self.room),
                    control::exchange::Message::DisableRaiseHands { issued_by: self.id },
                );
            }

            Event::Exchange(exchange::Message::Banned(participant)) => {
                if self.id == participant {
                    ctx.ws_send(ModerationEvent::Banned);
                    ctx.exit(Some(CloseCode::Normal));
                }
            }
            Event::Exchange(exchange::Message::Kicked(participant)) => {
                if self.id == participant {
                    ctx.ws_send(ModerationEvent::Kicked);
                    ctx.exit(Some(CloseCode::Normal));
                }
            }
            Event::Exchange(exchange::Message::Debriefed {
                kick_scope,
                issued_by,
            }) => {
                if kick_scope.kicks_role(ctx.role()) {
                    ctx.ws_send(ModerationEvent::SessionEnded { issued_by });
                    ctx.exit(Some(CloseCode::Normal));
                } else {
                    ctx.ws_send(ModerationEvent::DebriefingStarted { issued_by });
                }
            }
            Event::Exchange(exchange::Message::JoinedWaitingRoom(id)) => {
                if self.id == id {
                    return Ok(());
                }

                let control_data =
                    ControlState::from_redis(ctx.redis_conn(), self.room, id).await?;

                let mut module_data = ModulePeerData::new();
                module_data.insert(&control_data)?;

                ctx.ws_send(ModerationEvent::JoinedWaitingRoom(Participant {
                    id,
                    module_data,
                }));
            }
            Event::Exchange(exchange::Message::LeftWaitingRoom(id)) => {
                if self.id == id {
                    return Ok(());
                }

                ctx.ws_send(ModerationEvent::LeftWaitingRoom(AssociatedParticipant {
                    id,
                }));
            }
            Event::Exchange(exchange::Message::WaitingRoomEnableUpdated) => {
                let enabled =
                    storage::is_waiting_room_enabled(ctx.redis_conn(), self.room.room_id()).await?;

                if enabled {
                    ctx.ws_send(ModerationEvent::WaitingRoomEnabled);
                } else {
                    ctx.ws_send(ModerationEvent::WaitingRoomDisabled);
                }
            }
            Event::Ext(_) => unreachable!(),
        }

        Ok(())
    }

    async fn on_destroy(self, mut ctx: DestroyContext<'_>) {
        if ctx.destroy_room() {
            if let Err(e) = storage::delete_bans(ctx.redis_conn(), self.room.room_id()).await {
                log::error!("Failed to clean up bans list {}", e);
            }

            if let Err(e) =
                storage::delete_waiting_room_enabled(ctx.redis_conn(), self.room.room_id()).await
            {
                log::error!("Failed to clean up waiting room enabled flag {}", e);
            }

            if let Err(e) =
                storage::delete_raise_hands_enabled(ctx.redis_conn(), self.room.room_id()).await
            {
                log::error!("Failed to clean up raise hands enabled flag {}", e);
            }

            if let Err(e) =
                storage::delete_waiting_room(ctx.redis_conn(), self.room.room_id()).await
            {
                log::error!("Failed to clean up waiting room list {}", e);
            }

            if let Err(e) =
                storage::delete_waiting_room_accepted(ctx.redis_conn(), self.room.room_id()).await
            {
                log::error!("Failed to clean up accepted waiting room list {}", e);
            }
        }
    }

    async fn build_params(_init: SignalingModuleInitData) -> Result<Option<Self::Params>> {
        Ok(Some(()))
    }
}

#[cfg(test)]
mod tests {
    use serde_json::json;

    use super::*;

    #[test]
    fn frontend_data_for_moderator() {
        assert_eq!(
            serde_json::to_value(ModerationState {
                moderator_data: Some(ModeratorFrontendData {
                    waiting_room_enabled: true,
                    waiting_room_participants: vec![Participant {
                        id: ParticipantId::from_u128(1),
                        module_data: ModulePeerData::new()
                    }]
                }),
                raise_hands_enabled: false
            })
            .unwrap(),
            json!({
                "raise_hands_enabled": false,
                "waiting_room_enabled": true,
                "waiting_room_participants": [
                    {
                        "id": "00000000-0000-0000-0000-000000000001",
                    }
                ]
            })
        );
    }

    #[test]
    fn frontend_data_for_user() {
        assert_eq!(
            serde_json::to_value(ModerationState {
                moderator_data: None,
                raise_hands_enabled: false
            })
            .unwrap(),
            json!({
                "raise_hands_enabled": false,
            })
        );
    }
}
