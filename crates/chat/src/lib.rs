// SPDX-FileCopyrightText: OpenTalk GmbH <mail@opentalk.eu>
//
// SPDX-License-Identifier: EUPL-1.2

//! # Chat Module
//!
//! ## Functionality
//!
//! Issues timestamp and messageIds to incoming chat messages and forwards them to other participants in the room or group.

use anyhow::Result;
use database::Db;
use db_storage::groups::Group;
use r3dlock::Mutex;
use signaling_core::{
    control::{self, exchange},
    DestroyContext, Event, InitContext, ModuleContext, Participant, RedisConnection,
    SignalingModule, SignalingModuleInitData, SignalingRoomId,
};
use types::{
    core::{GroupId, GroupName, ParticipantId, Timestamp, UserId},
    signaling::{
        chat::{
            command::{ChatCommand, SendMessage},
            event::{ChatDisabled, ChatEnabled, ChatEvent, Error, HistoryCleared, MessageSent},
            peer_state::ChatPeerState,
            state::{ChatState, GroupHistory, PrivateHistory, StoredMessage},
            MessageId, Scope, NAMESPACE,
        },
        Role,
    },
};

use std::collections::HashMap;
use std::sync::Arc;

mod storage;

pub use storage::is_chat_enabled;

fn current_room_by_group_id(room_id: SignalingRoomId, group_id: GroupId) -> String {
    format!("room={room_id}:group={group_id}")
}

pub struct Chat {
    id: ParticipantId,
    room: SignalingRoomId,
    last_seen_timestamp_global: Option<Timestamp>,
    last_seen_timestamps_private: HashMap<ParticipantId, Timestamp>,
    last_seen_timestamps_group: HashMap<GroupName, Timestamp>,
    db: Arc<Db>,
    groups: Vec<Group>,
}

impl Chat {
    fn get_group(&self, name: &GroupName) -> Option<&Group> {
        self.groups.iter().find(|group| group.name == *name)
    }
}

#[async_trait::async_trait(?Send)]
trait ChatStateExt: Sized {
    async fn for_current_room_and_participant(
        redis_conn: &mut RedisConnection,
        room: SignalingRoomId,
        participant: ParticipantId,
        groups: &[Group],
    ) -> Result<Self>;
}

#[async_trait::async_trait(?Send)]
impl ChatStateExt for ChatState {
    async fn for_current_room_and_participant(
        redis_conn: &mut RedisConnection,
        room: SignalingRoomId,
        participant: ParticipantId,
        groups: &[Group],
    ) -> Result<Self> {
        let enabled = storage::is_chat_enabled(redis_conn, room.room_id()).await?;

        let room_history = storage::get_room_chat_history(redis_conn, room).await?;
        let mut groups_history = Vec::new();
        for group in groups {
            storage::add_participant_to_set(redis_conn, room, group.id, participant).await?;

            let history = storage::get_group_chat_history(redis_conn, room, group.id).await?;

            groups_history.push(GroupHistory {
                name: group.name.clone(),
                history,
            });
        }

        let mut private_history = Vec::new();
        let correspondents =
            storage::get_private_chat_correspondents_for_participant(redis_conn, room, participant)
                .await?;
        for correspondent in correspondents {
            let history =
                storage::get_private_chat_history(redis_conn, room, participant, correspondent)
                    .await?;
            private_history.push(PrivateHistory {
                correspondent,
                history,
            });
        }

        let last_seen_timestamp_global =
            storage::get_last_seen_timestamp_global(redis_conn, room, participant).await?;
        let last_seen_timestamps_private =
            storage::get_last_seen_timestamps_private(redis_conn, room, participant).await?;
        let last_seen_timestamps_group =
            storage::get_last_seen_timestamps_group(redis_conn, room, participant).await?;

        Ok(Self {
            room_history,
            enabled,
            groups_history,
            private_history,
            last_seen_timestamp_global,
            last_seen_timestamps_private,
            last_seen_timestamps_group,
        })
    }
}

#[async_trait::async_trait(? Send)]
impl SignalingModule for Chat {
    const NAMESPACE: &'static str = NAMESPACE;

    type Params = ();

    type Incoming = ChatCommand;
    type Outgoing = ChatEvent;
    type ExchangeMessage = ChatEvent;

    type ExtEvent = ();

    type FrontendData = ChatState;
    type PeerFrontendData = ChatPeerState;

    async fn init(
        mut ctx: InitContext<'_, Self>,
        _params: &Self::Params,
        _protocol: &'static str,
    ) -> Result<Option<Self>> {
        let id = ctx.participant_id();
        let room = ctx.room_id();

        let groups = if let Participant::User(user) = ctx.participant() {
            let mut conn = ctx.db().get_conn().await?;

            let groups = Group::get_all_for_user(&mut conn, user.id).await?;

            for group in &groups {
                log::debug!("Group: {}", group.id);

                ctx.add_exchange_binding(current_room_by_group_id(room, group.id));
            }
            groups
        } else {
            vec![]
        };

        Ok(Some(Self {
            id,
            room,
            db: ctx.db().clone(),
            groups,
            last_seen_timestamp_global: None,
            last_seen_timestamps_private: HashMap::new(),
            last_seen_timestamps_group: HashMap::new(),
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
                let module_frontend_data = ChatState::for_current_room_and_participant(
                    ctx.redis_conn(),
                    self.room,
                    self.id,
                    &self.groups,
                )
                .await?;
                self.last_seen_timestamp_global = module_frontend_data.last_seen_timestamp_global;
                self.last_seen_timestamps_private =
                    module_frontend_data.last_seen_timestamps_private.clone();
                self.last_seen_timestamps_group =
                    module_frontend_data.last_seen_timestamps_group.clone();

                *frontend_data = Some(module_frontend_data);

                // ==== Find other participant in our group ====
                let participant_ids: Vec<ParticipantId> =
                    participants.iter().map(|(id, _)| *id).collect();

                // Get all user_ids for each participant in the room
                let user_ids: Vec<Option<UserId>> =
                    control::storage::get_attribute_for_participants(
                        ctx.redis_conn(),
                        self.room,
                        "user_id",
                        &participant_ids,
                    )
                    .await?;

                // Filter out guest/bots and map each user-id to a participant id
                let participant_user_mappings: Vec<(UserId, ParticipantId)> = user_ids
                    .into_iter()
                    .enumerate()
                    .filter_map(|(i, opt_user_id)| {
                        opt_user_id.map(|user_id| (user_id, participant_ids[i]))
                    })
                    .collect();

                let db = self.db.clone();
                let self_groups = self.groups.clone();

                // Inquire the database about each user's groups
                let mut conn = db.get_conn().await?;

                let mut participant_to_common_groups_mappings = vec![];

                for (user_id, participant_id) in participant_user_mappings {
                    // Get the users groups
                    let groups = Group::get_all_for_user(&mut conn, user_id).await?;

                    // Intersect our groups and the groups of the user and collect their id/name
                    // as strings into a set
                    let common_groups = self_groups
                        .iter()
                        .filter(|self_group| groups.contains(self_group))
                        .map(|group| group.name.clone())
                        .collect();

                    participant_to_common_groups_mappings.push((participant_id, common_groups));
                }

                // Iterate over the result of the controller::block call and insert the common groups
                // into the PeerFrontendData
                for (participant_id, common_groups) in participant_to_common_groups_mappings {
                    if let Some(participant_frontend_data) = participants.get_mut(&participant_id) {
                        *participant_frontend_data = Some(ChatPeerState {
                            groups: common_groups,
                        });
                    } else {
                        log::error!("Got invalid participant id")
                    }
                }
            }
            Event::Leaving => {}
            Event::RaiseHand => {}
            Event::LowerHand => {}
            Event::ParticipantJoined(participant_id, peer_frontend_data) => {
                // Get user id of the joined participant
                let user_id: Option<UserId> = control::storage::get_attribute(
                    ctx.redis_conn(),
                    self.room,
                    participant_id,
                    "user_id",
                )
                .await?;

                if let Some(user_id) = user_id {
                    let db = self.db.clone();
                    let self_groups = self.groups.clone();

                    // Get the user's groups
                    let mut conn = db.get_conn().await?;

                    let groups = Group::get_all_for_user(&mut conn, user_id).await?;

                    // Intersect our groups and the groups of the user and collect their id/name
                    // as strings into a set
                    let common_groups = self_groups
                        .iter()
                        .filter(|self_group| groups.contains(self_group))
                        .map(|group| group.name.clone())
                        .collect();

                    *peer_frontend_data = Some(ChatPeerState {
                        groups: common_groups,
                    });
                }
            }
            Event::ParticipantLeft(_) => {}
            Event::ParticipantUpdated(_, _) => {}
            Event::RoleUpdated(_) => {}
            Event::WsMessage(ChatCommand::EnableChat) => {
                if ctx.role() != Role::Moderator {
                    ctx.ws_send(Error::InsufficientPermissions);
                    return Ok(());
                }

                storage::set_chat_enabled(ctx.redis_conn(), self.room.room_id(), true).await?;

                ctx.exchange_publish(
                    exchange::current_room_all_participants(self.room),
                    ChatEvent::ChatEnabled(ChatEnabled { issued_by: self.id }),
                );
            }
            Event::WsMessage(ChatCommand::DisableChat) => {
                if ctx.role() != Role::Moderator {
                    ctx.ws_send(Error::InsufficientPermissions);
                    return Ok(());
                }

                storage::set_chat_enabled(ctx.redis_conn(), self.room.room_id(), false).await?;

                ctx.exchange_publish(
                    exchange::current_room_all_participants(self.room),
                    ChatEvent::ChatDisabled(ChatDisabled { issued_by: self.id }),
                );
            }
            Event::WsMessage(ChatCommand::SendMessage(SendMessage { scope, mut content })) => {
                // Discard empty messages
                if content.is_empty() {
                    return Ok(());
                }

                let chat_enabled =
                    storage::is_chat_enabled(ctx.redis_conn(), self.room.room_id()).await?;

                if !chat_enabled {
                    ctx.ws_send(Error::ChatDisabled);
                    return Ok(());
                }

                // Limit message size
                let max_message_size = 4096;
                if content.len() > max_message_size {
                    let mut last_idx = 0;

                    for (i, _) in content.char_indices() {
                        if i > max_message_size {
                            break;
                        }
                        last_idx = i;
                    }

                    content.truncate(last_idx);
                }

                let source = self.id;

                match scope {
                    Scope::Private(target) => {
                        let out_message = MessageSent {
                            id: MessageId::generate(),
                            source,
                            content,
                            scope: Scope::Private(target),
                        };

                        let stored_msg = StoredMessage {
                            id: out_message.id,
                            source: out_message.source,
                            content: out_message.content.clone(),
                            scope: out_message.scope.clone(),
                            timestamp: ctx.timestamp(),
                        };

                        storage::add_private_chat_correspondents(
                            ctx.redis_conn(),
                            self.room,
                            self.id,
                            target,
                        )
                        .await?;

                        storage::add_message_to_private_chat_history(
                            ctx.redis_conn(),
                            self.room,
                            self.id,
                            target,
                            &stored_msg,
                        )
                        .await?;

                        ctx.exchange_publish(
                            exchange::current_room_by_participant_id(self.room, target),
                            ChatEvent::MessageSent(out_message.clone()),
                        );

                        ctx.ws_send(out_message);
                    }
                    Scope::Group(group_name) => {
                        if let Some(group) = self.get_group(&group_name) {
                            let out_message_contents = MessageSent {
                                id: MessageId::generate(),
                                source,
                                content,
                                scope: Scope::Group(group_name),
                            };

                            let stored_msg = StoredMessage {
                                id: out_message_contents.id,
                                source: out_message_contents.source,
                                content: out_message_contents.content.clone(),
                                scope: out_message_contents.scope.clone(),
                                timestamp: ctx.timestamp(),
                            };

                            storage::add_message_to_group_chat_history(
                                ctx.redis_conn(),
                                self.room,
                                group.id,
                                &stored_msg,
                            )
                            .await?;

                            let out_message = ChatEvent::MessageSent(out_message_contents);

                            ctx.exchange_publish(
                                current_room_by_group_id(self.room, group.id),
                                out_message,
                            )
                        }
                    }
                    Scope::Global => {
                        let out_message_contents = MessageSent {
                            id: MessageId::generate(),
                            source,
                            content,
                            scope: Scope::Global,
                        };

                        let stored_msg = StoredMessage {
                            id: out_message_contents.id,
                            source: out_message_contents.source,
                            content: out_message_contents.content.clone(),
                            scope: out_message_contents.scope.clone(),
                            timestamp: ctx.timestamp(),
                        };

                        storage::add_message_to_room_chat_history(
                            ctx.redis_conn(),
                            self.room,
                            &stored_msg,
                        )
                        .await?;

                        let out_message = ChatEvent::MessageSent(out_message_contents);

                        ctx.exchange_publish(
                            exchange::current_room_all_participants(self.room),
                            out_message,
                        );
                    }
                }
            }
            Event::WsMessage(ChatCommand::ClearHistory) => {
                if ctx.role() != Role::Moderator {
                    ctx.ws_send(Error::InsufficientPermissions);
                    return Ok(());
                }

                if let Err(e) = storage::delete_room_chat_history(ctx.redis_conn(), self.room).await
                {
                    log::error!("Failed to clear room chat history, {}", e);
                }

                ctx.exchange_publish(
                    exchange::current_room_all_participants(self.room),
                    ChatEvent::HistoryCleared(HistoryCleared { issued_by: self.id }),
                );
            }
            Event::WsMessage(ChatCommand::SetLastSeenTimestamp { scope, timestamp }) => {
                match scope {
                    Scope::Private(other_participant) => {
                        self.last_seen_timestamps_private
                            .insert(other_participant, timestamp);
                    }
                    Scope::Group(group) => {
                        if self.get_group(&group).is_some() {
                            self.last_seen_timestamps_group.insert(group, timestamp);
                        }
                    }
                    Scope::Global => {
                        self.last_seen_timestamp_global = Some(timestamp);
                    }
                };
            }
            Event::Exchange(msg) => {
                ctx.ws_send(msg);
            }
            Event::Ext(_) => {}
        }

        Ok(())
    }

    async fn on_destroy(self, mut ctx: DestroyContext<'_>) {
        // ==== Cleanup room ====
        if ctx.destroy_room() {
            if let Err(e) = storage::delete_room_chat_history(ctx.redis_conn(), self.room).await {
                log::error!("Failed to remove room chat history on room destroy, {}", e);
            }
            if let Err(e) =
                storage::delete_chat_enabled(ctx.redis_conn(), self.room.room_id()).await
            {
                log::error!("Failed to clean up chat enabled flag {}", e);
            }

            let correspondents =
                storage::get_private_chat_correspondents(ctx.redis_conn(), self.room)
                    .await
                    .unwrap_or_else(|e| {
                        log::error!("Failed to load room private chat correspondents, {}", e);
                        Default::default()
                    });
            for (a, b) in correspondents {
                if let Err(e) =
                    storage::delete_private_chat_history(ctx.redis_conn(), self.room, a, b).await
                {
                    log::error!("Failed to remove room private chat history, {}", e);
                }
            }

            let participants = control::storage::get_all_participants(ctx.redis_conn(), self.room)
                .await
                .unwrap_or_else(|e| {
                    log::error!("Failed to load room participants, {}", e);
                    Vec::new()
                });
            for participant in participants {
                if let Err(e) = storage::delete_last_seen_timestamp_global(
                    ctx.redis_conn(),
                    self.room,
                    participant,
                )
                .await
                {
                    log::error!(
                        "Failed to clean up last seen timestamp for global chat, {}",
                        e
                    );
                }
                if let Err(e) = storage::delete_last_seen_timestamps_group(
                    ctx.redis_conn(),
                    self.room,
                    participant,
                )
                .await
                {
                    log::error!(
                        "Failed to clean up last seen timestamps for group chats, {}",
                        e
                    );
                }
                if let Err(e) = storage::delete_last_seen_timestamps_private(
                    ctx.redis_conn(),
                    self.room,
                    participant,
                )
                .await
                {
                    log::error!(
                        "Failed to clean up last seen timestamps for private chats, {}",
                        e
                    );
                }
            }
        } else {
            if let Some(timestamp) = self.last_seen_timestamp_global {
                if let Err(e) = storage::set_last_seen_timestamp_global(
                    ctx.redis_conn(),
                    self.room,
                    self.id,
                    timestamp,
                )
                .await
                {
                    log::error!("Failed to set last seen timestamp for global chat, {}", e);
                }
            }
            if !self.last_seen_timestamps_group.is_empty() {
                let timestamps: Vec<(GroupName, Timestamp)> = self
                    .last_seen_timestamps_group
                    .iter()
                    .map(|(k, v)| (k.to_owned(), *v))
                    .collect();
                if let Err(e) = storage::set_last_seen_timestamps_group(
                    ctx.redis_conn(),
                    self.room,
                    self.id,
                    &timestamps,
                )
                .await
                {
                    log::error!("Failed to set last seen timestamps for group chat, {}", e);
                }
            }

            if !self.last_seen_timestamps_private.is_empty() {
                // ToRedisArgs not implemented for HashMap, so we copy the entries
                // for now.
                // See: https://github.com/redis-rs/redis-rs/issues/444
                let timestamps: Vec<(ParticipantId, Timestamp)> = self
                    .last_seen_timestamps_private
                    .iter()
                    .map(|(k, v)| (*k, *v))
                    .collect();
                if let Err(e) = storage::set_last_seen_timestamps_private(
                    ctx.redis_conn(),
                    self.room,
                    self.id,
                    &timestamps,
                )
                .await
                {
                    log::error!("Failed to set last seen timestamps for private chat, {}", e);
                }
            }
        }

        // ==== Cleanup groups ====
        for group in self.groups {
            let mut mutex = Mutex::new(storage::RoomGroupParticipantsLock {
                room: self.room,
                group: group.id,
            });

            let guard = match mutex.lock(ctx.redis_conn()).await {
                Ok(guard) => guard,
                Err(e) => {
                    log::error!(
                        "Failed to acquire lock to cleanup group {:?}, {}",
                        group.id,
                        e
                    );

                    continue;
                }
            };

            let remove_history = match storage::remove_participant_from_set(
                &guard,
                ctx.redis_conn(),
                self.room,
                group.id,
                self.id,
            )
            .await
            {
                Ok(n) => n == 0,
                Err(e) => {
                    log::error!("Failed to remove participant from group set, {}", e);
                    false
                }
            };

            if remove_history {
                if let Err(e) =
                    storage::delete_group_chat_history(ctx.redis_conn(), self.room, group.id).await
                {
                    log::error!("Failed to remove room group chat history, {}", e);
                }
            }

            if let Err(e) = guard.unlock(ctx.redis_conn()).await {
                log::error!("Failed to unlock r3dlock, {}", e);
            }
        }
    }

    async fn build_params(_init: SignalingModuleInitData) -> Result<Option<Self::Params>> {
        Ok(Some(()))
    }
}
