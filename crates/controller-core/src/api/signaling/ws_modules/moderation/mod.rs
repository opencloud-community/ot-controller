// SPDX-FileCopyrightText: OpenTalk GmbH <mail@opentalk.eu>
//
// SPDX-License-Identifier: EUPL-1.2

use std::iter::zip;

use actix_http::ws::CloseCode;
use opentalk_signaling_core::{
    control::{self, storage::ControlStorage as _, ControlStateExt as _},
    DestroyContext, Event, InitContext, ModuleContext, RedisConnection, SerdeJsonSnafu,
    SignalingModule, SignalingModuleError, SignalingModuleInitData, SignalingRoomId,
};
pub use opentalk_types::signaling::moderation::NAMESPACE;
use opentalk_types::{
    core::{ParticipantId, ParticipationKind, RoomId, UserId},
    signaling::{
        control::{
            state::ControlState, AssociatedParticipant, Participant, Reason, WaitingRoomState,
        },
        moderation::{
            command::ModerationCommand,
            event::{Error, ModerationEvent},
            state::{ModerationState, ModeratorFrontendData},
        },
        ModulePeerData, Role,
    },
};
use snafu::{Report, ResultExt};

use self::storage::ModerationStorage as _;
use crate::api::signaling::{trim_display_name, ws::ModuleContextExt};

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
) -> Result<Vec<Participant>, SignalingModuleError> {
    let mut waiting_room = Vec::with_capacity(list.len());

    for id in list {
        let control_data =
            ControlState::from_redis(redis_conn, SignalingRoomId::new(room_id, None), *id).await?;
        // .whatever_context::<&str, SignalingModuleError>("Failed to get control state")?;

        let mut module_data = ModulePeerData::new();
        module_data.insert(&control_data).context(SerdeJsonSnafu {
            message: "Failed to serialize control state",
        })?;
        module_data
            .insert(&waiting_room_state)
            .context(SerdeJsonSnafu {
                message: "Failed to serialize waiting room state",
            })?;

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
) -> Result<(), SignalingModuleError> {
    ctx.redis_conn()
        .set_waiting_room_enabled(room_id, enabled)
        .await?;

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
    ) -> Result<Option<Self>, SignalingModuleError> {
        Ok(Some(Self {
            room: ctx.room_id(),
            id: ctx.participant_id(),
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
                let moderator_data = if ctx.role() == Role::Moderator {
                    let waiting_room_enabled = ctx
                        .redis_conn()
                        .is_waiting_room_enabled(self.room.room_id())
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

                let raise_hands_enabled = ctx
                    .redis_conn()
                    .is_raise_hands_enabled(self.room.room_id())
                    .await?;

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
                    ctx.ws_send(Error::InsufficientPermissions);
                    return Ok(());
                }

                storage::waiting_room_accepted_remove(
                    ctx.redis_conn(),
                    self.room.room_id(),
                    target,
                )
                .await?;

                let user_id: Option<UserId> = ctx
                    .redis_conn()
                    .get_attribute(self.room, target, "user_id")
                    .await?;

                if let Some(user_id) = user_id {
                    ctx.redis_conn()
                        .ban_user(self.room.room_id(), user_id)
                        .await?;
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
                    ctx.ws_send(Error::InsufficientPermissions);
                    return Ok(());
                }

                // Enforce the participant to enter the waiting room (if enabled) on next rejoin
                ctx.redis_conn()
                    .set_skip_waiting_room_with_expiry(target, false)
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
            Event::WsMessage(ModerationCommand::SendToWaitingRoom { target }) => {
                if ctx.role() != Role::Moderator {
                    ctx.ws_send(Error::InsufficientPermissions);
                    return Ok(());
                }

                // The room owner cannot be sent to the waiting room
                if ctx
                    .redis_conn()
                    .get_attribute(self.room, target, "is_room_owner")
                    .await?
                {
                    ctx.ws_send(Error::CannotSendRoomOwnerToWaitingRoom);
                    return Ok(());
                }

                if !ctx
                    .redis_conn
                    .is_waiting_room_enabled(self.room.room_id())
                    .await?
                {
                    set_waiting_room_enabled(&mut ctx, self.room.room_id(), true).await?;
                }

                // Enforce the participant to enter the waiting room (if enabled) on next rejoin
                ctx.redis_conn()
                    .set_skip_waiting_room_with_expiry(target, false)
                    .await?;

                storage::waiting_room_accepted_remove(
                    ctx.redis_conn(),
                    self.room.room_id(),
                    target,
                )
                .await?;

                ctx.exchange_publish(
                    control::exchange::current_room_by_participant_id(self.room, target),
                    exchange::Message::SentToWaitingRoom(target),
                );
            }

            Event::WsMessage(ModerationCommand::Debrief(kick_scope)) => {
                if ctx.role() != Role::Moderator {
                    ctx.ws_send(Error::InsufficientPermissions);
                    return Ok(());
                }

                // Remove all debriefed participants from the waiting-room-accepted set
                let all_participants =
                    Vec::from_iter(ctx.redis_conn().get_all_participants(self.room).await?);
                let all_participants_role: Vec<Option<Role>> = ctx
                    .redis_conn()
                    .get_attribute_for_participants(self.room, "role", &all_participants)
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
                        ctx.redis_conn()
                            .set_skip_waiting_room_with_expiry(id, false)
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

            Event::WsMessage(ModerationCommand::ChangeDisplayName { new_name, target }) => {
                if ctx.role() != Role::Moderator {
                    ctx.ws_send(Error::InsufficientPermissions);
                    return Ok(());
                }

                let kind: Option<ParticipationKind> = ctx
                    .redis_conn()
                    .get_attribute(self.room, target, "kind")
                    .await?;

                if !matches!(
                    kind,
                    Some(ParticipationKind::Guest) | Some(ParticipationKind::Sip)
                ) {
                    ctx.ws_send(Error::CannotChangeNameOfRegisteredUsers);
                    return Ok(());
                }

                let new_name = trim_display_name(new_name);

                if new_name.is_empty() || new_name.len() > 100 {
                    ctx.ws_send(Error::InvalidDisplayName);
                    return Ok(());
                }

                ctx.redis_conn()
                    .set_attribute(self.room, target, "display_name", new_name)
                    .await?;

                ctx.exchange_publish(
                    control::exchange::current_room_by_participant_id(self.room, target),
                    exchange::Message::DisplayNameChanged,
                );
            }

            Event::WsMessage(ModerationCommand::EnableWaitingRoom) => {
                if ctx.role() != Role::Moderator {
                    ctx.ws_send(Error::InsufficientPermissions);
                    return Ok(());
                }

                set_waiting_room_enabled(&mut ctx, self.room.room_id(), true).await?;
            }
            Event::WsMessage(ModerationCommand::DisableWaitingRoom) => {
                if ctx.role() != Role::Moderator {
                    ctx.ws_send(Error::InsufficientPermissions);
                    return Ok(());
                }

                set_waiting_room_enabled(&mut ctx, self.room.room_id(), false).await?;
            }
            Event::WsMessage(ModerationCommand::Accept { target }) => {
                if ctx.role() != Role::Moderator {
                    ctx.ws_send(Error::InsufficientPermissions);
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
            Event::WsMessage(ModerationCommand::ResetRaisedHands { target }) => {
                if ctx.role() != Role::Moderator {
                    ctx.ws_send(Error::InsufficientPermissions);
                    return Ok(());
                }

                if let Some(target) = target {
                    ctx.exchange_publish_control(
                        control::exchange::current_room_by_participant_id(self.room, target),
                        control::exchange::Message::ResetRaisedHands { issued_by: self.id },
                    );
                } else {
                    ctx.exchange_publish_control(
                        control::exchange::current_room_all_participants(self.room),
                        control::exchange::Message::ResetRaisedHands { issued_by: self.id },
                    );
                }
            }

            Event::WsMessage(ModerationCommand::EnableRaiseHands) => {
                if ctx.role() != Role::Moderator {
                    ctx.ws_send(Error::InsufficientPermissions);
                    return Ok(());
                }

                ctx.redis_conn()
                    .set_raise_hands_enabled(self.room.room_id(), true)
                    .await?;

                ctx.exchange_publish_control(
                    control::exchange::current_room_all_participants(self.room),
                    control::exchange::Message::EnableRaiseHands { issued_by: self.id },
                );
            }

            Event::WsMessage(ModerationCommand::DisableRaiseHands) => {
                if ctx.role() != Role::Moderator {
                    ctx.ws_send(Error::InsufficientPermissions);
                    return Ok(());
                }

                ctx.redis_conn()
                    .set_raise_hands_enabled(self.room.room_id(), false)
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
            Event::Exchange(exchange::Message::SentToWaitingRoom(participant)) => {
                if self.id == participant {
                    ctx.ws_send(ModerationEvent::SentToWaitingRoom);

                    ctx.exchange_publish_control(
                        control::exchange::current_room_all_participants(self.room),
                        control::exchange::Message::Left {
                            id: self.id,
                            reason: Reason::SentToWaitingRoom,
                        },
                    );
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
            Event::Exchange(exchange::Message::DisplayNameChanged) => ctx.invalidate_data(),
            Event::Exchange(exchange::Message::JoinedWaitingRoom(id)) => {
                if self.id == id {
                    return Ok(());
                }

                let control_data =
                    ControlState::from_redis(ctx.redis_conn(), self.room, id).await?;

                let mut module_data = ModulePeerData::new();
                module_data.insert(&control_data).context(SerdeJsonSnafu {
                    message: "Failed to serialize control state",
                })?;

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
                let enabled = ctx
                    .redis_conn()
                    .is_waiting_room_enabled(self.room.room_id())
                    .await?;

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
            if let Err(e) = ctx.redis_conn().delete_user_bans(self.room.room_id()).await {
                log::error!("Failed to clean up bans list {}", Report::from_error(e));
            }

            if let Err(e) = ctx
                .redis_conn()
                .delete_waiting_room_enabled(self.room.room_id())
                .await
            {
                log::error!(
                    "Failed to clean up waiting room enabled flag {}",
                    Report::from_error(e)
                );
            }

            if let Err(e) = ctx
                .redis_conn()
                .delete_raise_hands_enabled(self.room.room_id())
                .await
            {
                log::error!(
                    "Failed to clean up raise hands enabled flag {}",
                    Report::from_error(e)
                );
            }

            if let Err(e) =
                storage::delete_waiting_room(ctx.redis_conn(), self.room.room_id()).await
            {
                log::error!(
                    "Failed to clean up waiting room list {}",
                    Report::from_error(e)
                );
            }

            if let Err(e) =
                storage::delete_waiting_room_accepted(ctx.redis_conn(), self.room.room_id()).await
            {
                log::error!(
                    "Failed to clean up accepted waiting room list {}",
                    Report::from_error(e)
                );
            }
        }
    }

    async fn build_params(
        _init: SignalingModuleInitData,
    ) -> Result<Option<Self::Params>, SignalingModuleError> {
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
