// SPDX-FileCopyrightText: OpenTalk GmbH <mail@opentalk.eu>
//
// SPDX-License-Identifier: EUPL-1.2

use std::collections::{HashMap, HashSet};

use opentalk_signaling_core::SignalingRoomId;
use opentalk_types::{
    core::{GroupName, ParticipantId, RoomId, Timestamp},
    signaling::chat::state::StoredMessage,
};
use opentalk_types_common::users::GroupId;

use crate::ParticipantPair;

#[derive(Debug, Clone, Default)]
pub(super) struct MemoryChatState {
    room_history: HashMap<SignalingRoomId, Vec<StoredMessage>>,
    group_history: HashMap<(SignalingRoomId, GroupId), Vec<StoredMessage>>,
    private_history: HashMap<(SignalingRoomId, ParticipantPair), Vec<StoredMessage>>,
    chats_enabled: HashMap<RoomId, bool>,
    last_seen_timestamps_private:
        HashMap<SignalingRoomId, HashMap<ParticipantId, HashMap<ParticipantId, Timestamp>>>,
    last_seen_timestamps_group:
        HashMap<SignalingRoomId, HashMap<ParticipantId, HashMap<GroupName, Timestamp>>>,
    last_seen_timestamps_global: HashMap<SignalingRoomId, HashMap<ParticipantId, Timestamp>>,
    private_correspondents: HashMap<SignalingRoomId, HashSet<ParticipantPair>>,
    group_participants: HashMap<(SignalingRoomId, GroupId), HashSet<ParticipantId>>,
}

impl MemoryChatState {
    #[cfg(test)]
    pub(super) fn reset(&mut self) {
        *self = Self::default();
    }

    pub(super) fn get_room_history(&self, room: SignalingRoomId) -> Vec<StoredMessage> {
        self.room_history.get(&room).cloned().unwrap_or_default()
    }

    pub(super) fn add_message_to_room_history(
        &mut self,
        room: SignalingRoomId,
        message: &StoredMessage,
    ) {
        self.room_history
            .entry(room)
            .or_default()
            .push(message.clone());
    }

    pub(super) fn delete_room_history(&mut self, room: SignalingRoomId) {
        self.room_history.remove(&room);
    }

    pub(super) fn set_chat_enabled(&mut self, room: RoomId, enabled: bool) {
        self.chats_enabled.insert(room, enabled);
    }

    pub(super) fn is_chat_enabled(&self, room: RoomId) -> bool {
        self.chats_enabled.get(&room).cloned().unwrap_or(true)
    }

    pub(super) fn delete_chat_enabled(&mut self, room: RoomId) {
        self.chats_enabled.remove(&room);
    }

    pub(super) fn set_last_seen_timestamps_private(
        &mut self,
        room: SignalingRoomId,
        participant: ParticipantId,
        timestamps: &[(ParticipantId, Timestamp)],
    ) {
        self.last_seen_timestamps_private
            .entry(room)
            .or_default()
            .entry(participant)
            .or_default()
            .extend(timestamps.iter().cloned());
    }

    pub(super) fn get_last_seen_timestamps_private(
        &self,
        room: SignalingRoomId,
        participant: ParticipantId,
    ) -> HashMap<ParticipantId, Timestamp> {
        self.last_seen_timestamps_private
            .get(&room)
            .and_then(|participants| participants.get(&participant).cloned())
            .unwrap_or_default()
    }

    pub(super) fn delete_last_seen_timestamps_private(
        &mut self,
        room: SignalingRoomId,
        participant: ParticipantId,
    ) {
        self.last_seen_timestamps_private
            .get_mut(&room)
            .and_then(|participants| participants.remove(&participant));
        if matches!(self.last_seen_timestamps_private.get(&room), Some(p) if p.is_empty()) {
            self.last_seen_timestamps_private.remove(&room);
        }
    }

    pub(super) fn set_last_seen_timestamps_group(
        &mut self,
        room: SignalingRoomId,
        participant: ParticipantId,
        timestamps: &[(GroupName, Timestamp)],
    ) {
        self.last_seen_timestamps_group
            .entry(room)
            .or_default()
            .entry(participant)
            .or_default()
            .extend(timestamps.iter().cloned());
    }

    pub(super) fn get_last_seen_timestamps_group(
        &self,
        room: SignalingRoomId,
        participant: ParticipantId,
    ) -> HashMap<GroupName, Timestamp> {
        self.last_seen_timestamps_group
            .get(&room)
            .and_then(|participants| participants.get(&participant).cloned())
            .unwrap_or_default()
    }

    pub(super) fn delete_last_seen_timestamps_group(
        &mut self,
        room: SignalingRoomId,
        participant: ParticipantId,
    ) {
        self.last_seen_timestamps_group
            .get_mut(&room)
            .and_then(|participants| participants.remove(&participant));
        if matches!(self.last_seen_timestamps_group.get(&room), Some(p) if p.is_empty()) {
            self.last_seen_timestamps_group.remove(&room);
        }
    }

    pub(super) fn set_last_seen_timestamp_global(
        &mut self,
        room: SignalingRoomId,
        participant: ParticipantId,
        timestamp: Timestamp,
    ) {
        self.last_seen_timestamps_global
            .entry(room)
            .or_default()
            .entry(participant)
            .or_insert(timestamp);
    }

    pub(super) fn get_last_seen_timestamp_global(
        &self,
        room: SignalingRoomId,
        participant: ParticipantId,
    ) -> Option<Timestamp> {
        self.last_seen_timestamps_global
            .get(&room)
            .and_then(|participants| participants.get(&participant).cloned())
    }

    pub(super) fn delete_last_seen_timestamp_global(
        &mut self,
        room: SignalingRoomId,
        participant: ParticipantId,
    ) {
        self.last_seen_timestamps_global
            .get_mut(&room)
            .and_then(|participants| participants.remove(&participant));
        if matches!(self.last_seen_timestamps_global.get(&room), Some(p) if p.is_empty()) {
            self.last_seen_timestamps_global.remove(&room);
        }
    }

    pub(super) fn add_private_chat_correspondents(
        &mut self,
        room: SignalingRoomId,
        participant_one: ParticipantId,
        participant_two: ParticipantId,
    ) {
        self.private_correspondents
            .entry(room)
            .or_default()
            .insert(ParticipantPair::new(participant_one, participant_two));
    }

    pub(super) fn delete_private_chat_correspondents(&mut self, room: SignalingRoomId) {
        self.private_correspondents.remove(&room);
    }

    pub(super) fn get_private_chat_correspondents(
        &self,
        room: SignalingRoomId,
    ) -> HashSet<ParticipantPair> {
        self.private_correspondents
            .get(&room)
            .cloned()
            .unwrap_or_default()
    }

    pub(super) fn get_group_chat_history(
        &self,
        room: SignalingRoomId,
        group: GroupId,
    ) -> Vec<StoredMessage> {
        self.group_history
            .get(&(room, group))
            .cloned()
            .unwrap_or_default()
    }

    pub(super) fn add_message_to_group_chat_history(
        &mut self,
        room: SignalingRoomId,
        group: GroupId,
        message: &StoredMessage,
    ) {
        self.group_history
            .entry((room, group))
            .or_default()
            .push(message.clone());
    }

    pub(super) fn delete_group_chat_history(&mut self, room: SignalingRoomId, group: GroupId) {
        self.group_history.remove(&(room, group));
    }

    pub(super) fn get_private_chat_history(
        &self,
        room: SignalingRoomId,
        participant_one: ParticipantId,
        participant_two: ParticipantId,
    ) -> Vec<StoredMessage> {
        self.private_history
            .get(&(room, ParticipantPair::new(participant_one, participant_two)))
            .cloned()
            .unwrap_or_default()
    }

    pub(super) fn add_message_to_private_chat_history(
        &mut self,
        room: SignalingRoomId,
        participant_one: ParticipantId,
        participant_two: ParticipantId,
        message: &StoredMessage,
    ) {
        self.private_history
            .entry((room, ParticipantPair::new(participant_one, participant_two)))
            .or_default()
            .push(message.clone());
    }

    pub(super) fn delete_private_chat_history(
        &mut self,
        room: SignalingRoomId,
        participant_one: ParticipantId,
        participant_two: ParticipantId,
    ) {
        self.private_history
            .remove(&(room, ParticipantPair::new(participant_one, participant_two)));
    }

    pub(super) fn add_participant_to_group(
        &mut self,
        room: SignalingRoomId,
        group: GroupId,
        participant: ParticipantId,
    ) {
        self.group_participants
            .entry((room, group))
            .or_default()
            .insert(participant);
    }

    pub(super) fn remove_participant_from_group(
        &mut self,
        room: SignalingRoomId,
        group: GroupId,
        participant: ParticipantId,
    ) {
        let group_is_empty = self
            .group_participants
            .get_mut(&(room, group))
            .map(|participants| {
                participants.remove(&participant);
                participants.is_empty()
            })
            .unwrap_or_default();

        if group_is_empty {
            self.group_participants.remove(&(room, group));
        }
    }
}
