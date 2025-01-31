// SPDX-FileCopyrightText: OpenTalk GmbH <mail@opentalk.eu>
//
// SPDX-License-Identifier: EUPL-1.2

//! # Chat Module
//!
//! ## Functionality
//!
//! Issues timestamp and messageIds to incoming chat messages and forwards them to other participants in the room or group.

use std::{
    collections::{BTreeMap, BTreeSet},
    sync::Arc,
};

use either::Either;
use opentalk_database::Db;
use opentalk_db_storage::groups::Group;
use opentalk_signaling_core::{
    control::{
        exchange,
        storage::{ControlStorageParticipantAttributes as _, USER_ID},
    },
    CleanupScope, DestroyContext, Event, InitContext, LockError, ModuleContext, Participant,
    RoomLockingProvider as _, SignalingModule, SignalingModuleError, SignalingModuleInitData,
    SignalingRoomId, VolatileStorage,
};
use opentalk_types_common::{
    modules::ModuleId,
    time::Timestamp,
    users::{GroupId, GroupName, UserId},
};
use opentalk_types_signaling::{ParticipantId, Role};
use opentalk_types_signaling_chat::{
    command::{ChatCommand, SendMessage, SetLastSeenTimestamp},
    event::{ChatDisabled, ChatEnabled, ChatEvent, Error, HistoryCleared, MessageSent},
    peer_state::ChatPeerState,
    state::{ChatState, GroupHistory, PrivateHistory, StoredMessage},
    MessageId, Scope, MODULE_ID,
};
use snafu::Report;

mod participant_pair;
mod storage;

use participant_pair::ParticipantPair;
use storage::ChatStorage;

fn current_room_by_group_id(room_id: SignalingRoomId, group_id: GroupId) -> String {
    format!("room={room_id}:group={group_id}")
}

pub struct Chat {
    id: ParticipantId,
    room: SignalingRoomId,
    last_seen_timestamp_global: Option<Timestamp>,
    last_seen_timestamps_private: BTreeMap<ParticipantId, Timestamp>,
    last_seen_timestamps_group: BTreeMap<GroupName, Timestamp>,
    db: Arc<Db>,
    groups: Vec<Group>,
}

impl Chat {
    fn get_group(&self, name: &GroupName) -> Option<&Group> {
        self.groups.iter().find(|group| group.name == *name)
    }

    async fn cleanup_room(ctx: &mut DestroyContext<'_>, signaling_room_id: SignalingRoomId) {
        if let Err(e) = ctx
            .volatile
            .storage()
            .delete_room_history(signaling_room_id)
            .await
        {
            log::error!(
                "Failed to remove room chat history on room destroy, {}",
                Report::from_error(e)
            );
        }

        if let Err(e) = ctx
            .volatile
            .storage()
            .delete_private_chat_correspondents(signaling_room_id)
            .await
        {
            log::error!(
                "Failed to remove room chat correspondents on room destroy, {}",
                Report::from_error(e)
            );
        }

        let correspondents = ctx
            .volatile
            .storage()
            .get_private_chat_correspondents(signaling_room_id)
            .await
            .unwrap_or_else(|e| {
                log::error!(
                    "Failed to load room private chat correspondents, {}",
                    Report::from_error(e)
                );
                Default::default()
            });
        for (a, b) in correspondents.iter().map(ParticipantPair::as_tuple) {
            if let Err(e) = ctx
                .volatile
                .storage()
                .delete_private_chat_history(signaling_room_id, a, b)
                .await
            {
                log::error!(
                    "Failed to remove room private chat history, {}",
                    Report::from_error(e)
                );
            }
        }

        let participants = ctx
            .volatile
            .storage()
            .get_all_participants(signaling_room_id)
            .await
            .unwrap_or_else(|e| {
                log::error!(
                    "Failed to load room participants, {}",
                    Report::from_error(e)
                );
                BTreeSet::new()
            });

        for participant in participants {
            if let Err(e) = ctx
                .volatile
                .storage()
                .delete_last_seen_timestamp_global(signaling_room_id, participant)
                .await
            {
                log::error!(
                    "Failed to clean up last seen timestamp for global chat, {}",
                    e
                );
            }
            if let Err(e) = ctx
                .volatile
                .storage()
                .delete_last_seen_timestamps_group(signaling_room_id, participant)
                .await
            {
                log::error!(
                    "Failed to clean up last seen timestamps for group chats, {}",
                    e
                );
            }
            if let Err(e) = ctx
                .volatile
                .storage()
                .delete_last_seen_timestamps_private(signaling_room_id, participant)
                .await
            {
                log::error!(
                    "Failed to clean up last seen timestamps for private chats, {}",
                    e
                );
            }
        }
    }

    async fn cleanup_global_room(&self, ctx: &mut DestroyContext<'_>) {
        if let Err(e) = ctx
            .volatile
            .storage()
            .delete_chat_enabled(self.room.room_id())
            .await
        {
            log::error!(
                "Failed to clean up chat enabled flag {}",
                Report::from_error(e)
            );
        }
    }
}

#[async_trait::async_trait(?Send)]
trait ChatStateExt: Sized {
    async fn for_current_room_and_participant(
        storage: &mut dyn ChatStorage,
        room: SignalingRoomId,
        participant: ParticipantId,
        groups: &[Group],
    ) -> Result<Self, SignalingModuleError>;
}

#[async_trait::async_trait(?Send)]
impl ChatStateExt for ChatState {
    async fn for_current_room_and_participant(
        storage: &mut dyn ChatStorage,
        room: SignalingRoomId,
        participant: ParticipantId,
        groups: &[Group],
    ) -> Result<Self, SignalingModuleError> {
        let enabled = storage.is_chat_enabled(room.room_id()).await?;

        let room_history = storage.get_room_history(room).await?;
        let mut groups_history = Vec::new();
        for group in groups {
            storage
                .add_participant_to_group(room, group.id, participant)
                .await?;

            let history = storage.get_group_chat_history(room, group.id).await?;

            groups_history.push(GroupHistory {
                name: group.name.clone(),
                history,
            });
        }

        let mut private_history = Vec::new();
        let correspondents = storage
            .get_private_chat_correspondents_for_participant(room, participant)
            .await?;
        for correspondent in correspondents {
            let history = storage
                .get_private_chat_history(room, participant, correspondent)
                .await?;
            private_history.push(PrivateHistory {
                correspondent,
                history,
            });
        }

        let last_seen_timestamp_global = storage
            .get_last_seen_timestamp_global(room, participant)
            .await?;
        let last_seen_timestamps_private = storage
            .get_last_seen_timestamps_private(room, participant)
            .await?;
        let last_seen_timestamps_group = storage
            .get_last_seen_timestamps_group(room, participant)
            .await?;

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

trait ChatStorageProvider {
    fn storage(&mut self) -> &mut dyn ChatStorage;
}

impl ChatStorageProvider for VolatileStorage {
    fn storage(&mut self) -> &mut dyn ChatStorage {
        match self.as_mut() {
            Either::Left(v) => v,
            Either::Right(v) => v,
        }
    }
}

#[async_trait::async_trait(? Send)]
impl SignalingModule for Chat {
    const NAMESPACE: ModuleId = MODULE_ID;

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
    ) -> Result<Option<Self>, SignalingModuleError> {
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
            last_seen_timestamps_private: BTreeMap::new(),
            last_seen_timestamps_group: BTreeMap::new(),
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
                let module_frontend_data = ChatState::for_current_room_and_participant(
                    ctx.volatile.storage(),
                    self.room,
                    self.id,
                    &self.groups,
                )
                .await?;
                self.last_seen_timestamp_global = module_frontend_data.last_seen_timestamp_global;
                self.last_seen_timestamps_private
                    .clone_from(&module_frontend_data.last_seen_timestamps_private);
                self.last_seen_timestamps_group
                    .clone_from(&module_frontend_data.last_seen_timestamps_group);

                *frontend_data = Some(module_frontend_data);

                // ==== Find other participant in our group ====
                let participant_ids: Vec<ParticipantId> =
                    participants.iter().map(|(id, _)| *id).collect();

                // Get all user_ids for each participant in the room
                let user_ids: Vec<Option<UserId>> = ctx
                    .volatile
                    .storage()
                    .get_local_attribute_for_participants(&participant_ids, self.room, USER_ID)
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
            Event::Leaving => {
                // acquire room lock
                let room_guard = match ctx.volatile.room_locking().lock_room(self.room).await {
                    Ok(guard) => guard,
                    Err(e @ LockError::Locked) => {
                        log::error!(
                            "Failed to acquire r3dlock, contention too high, {}",
                            &Report::from_error(e)
                        );
                        return Ok(());
                    }
                    Err(e) => {
                        log::error!("Failed to acquire r3dlock, {}", Report::from_error(e));
                        return Ok(());
                    }
                };

                if let Some(timestamp) = self.last_seen_timestamp_global {
                    if let Err(e) = ctx
                        .volatile
                        .storage()
                        .set_last_seen_timestamp_global(self.room, self.id, timestamp)
                        .await
                    {
                        log::error!(
                            "Failed to set last seen timestamp for global chat, {}",
                            Report::from_error(e)
                        );
                    }
                }

                if !self.last_seen_timestamps_group.is_empty() {
                    let timestamps: Vec<(GroupName, Timestamp)> = self
                        .last_seen_timestamps_group
                        .iter()
                        .map(|(k, v)| (k.to_owned(), *v))
                        .collect();
                    if let Err(e) = ctx
                        .volatile
                        .storage()
                        .set_last_seen_timestamps_group(self.room, self.id, &timestamps)
                        .await
                    {
                        log::error!(
                            "Failed to set last seen timestamps for group chat, {}",
                            Report::from_error(e)
                        );
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
                    if let Err(e) = ctx
                        .volatile
                        .storage()
                        .set_last_seen_timestamps_private(self.room, self.id, &timestamps)
                        .await
                    {
                        log::error!(
                            "Failed to set last seen timestamps for private chat, {}",
                            Report::from_error(e)
                        );
                    }
                }

                for group in &self.groups {
                    ctx.volatile
                        .storage()
                        .remove_participant_from_group(self.room, group.id, self.id)
                        .await;
                }

                if let Err(e) = ctx.volatile.room_locking().unlock_room(room_guard).await {
                    log::error!("Failed to unlock room , {}", Report::from_error(e));
                }
            }
            Event::RaiseHand => {}
            Event::LowerHand => {}
            Event::ParticipantJoined(participant_id, peer_frontend_data) => {
                // Get user id of the joined participant
                let user_id: Option<UserId> = ctx
                    .volatile
                    .storage()
                    .get_local_attribute(participant_id, self.room, USER_ID)
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

                ctx.volatile
                    .storage()
                    .set_chat_enabled(self.room.room_id(), true)
                    .await?;

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

                ctx.volatile
                    .storage()
                    .set_chat_enabled(self.room.room_id(), false)
                    .await?;

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

                let chat_enabled = ctx
                    .volatile
                    .storage()
                    .is_chat_enabled(self.room.room_id())
                    .await?;

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

                        ctx.volatile
                            .storage()
                            .add_private_chat_correspondents(self.room, self.id, target)
                            .await?;

                        ctx.volatile
                            .storage()
                            .add_message_to_private_chat_history(
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

                            ctx.volatile
                                .storage()
                                .add_message_to_group_chat_history(self.room, group.id, &stored_msg)
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

                        ctx.volatile
                            .storage()
                            .add_message_to_room_history(self.room, &stored_msg)
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

                if let Err(e) = ctx.volatile.storage().delete_room_history(self.room).await {
                    log::error!(
                        "Failed to clear room chat history, {}",
                        Report::from_error(e)
                    );
                }

                ctx.exchange_publish(
                    exchange::current_room_all_participants(self.room),
                    ChatEvent::HistoryCleared(HistoryCleared { issued_by: self.id }),
                );
            }
            Event::WsMessage(ChatCommand::SetLastSeenTimestamp(SetLastSeenTimestamp {
                scope,
                timestamp,
            })) => {
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
        match ctx.cleanup_scope {
            CleanupScope::None => {}
            CleanupScope::Local => Chat::cleanup_room(&mut ctx, self.room).await,
            CleanupScope::Global => {
                if self.room.breakout_room_id().is_some() {
                    // Cleanup state for the main room
                    Chat::cleanup_room(&mut ctx, SignalingRoomId::new(self.room.room_id(), None))
                        .await;
                }

                Chat::cleanup_room(&mut ctx, self.room).await;

                self.cleanup_global_room(&mut ctx).await;
            }
        }
    }

    async fn build_params(
        _init: SignalingModuleInitData,
    ) -> Result<Option<Self::Params>, SignalingModuleError> {
        Ok(Some(()))
    }
}
