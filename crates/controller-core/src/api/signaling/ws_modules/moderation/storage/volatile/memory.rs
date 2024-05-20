// SPDX-FileCopyrightText: OpenTalk GmbH <mail@opentalk.eu>
//
// SPDX-License-Identifier: EUPL-1.2

use std::collections::{BTreeSet, HashMap, HashSet};

use opentalk_types::core::{ParticipantId, RoomId, UserId};

#[derive(Debug, Clone, Default)]
pub(super) struct MemoryModerationState {
    banned_users: HashMap<RoomId, HashSet<UserId>>,
    waiting_room_enabled: HashMap<RoomId, bool>,
    raise_hands_enabled: HashMap<RoomId, bool>,
    waiting_room_participants: HashMap<RoomId, HashSet<ParticipantId>>,
    waiting_room_accepted_participants: HashMap<RoomId, HashSet<ParticipantId>>,
}

impl MemoryModerationState {
    #[cfg(test)]
    pub(super) fn reset(&mut self) {
        *self = Self::default();
    }

    pub(super) fn ban_user(&mut self, room: RoomId, user: UserId) {
        self.banned_users.entry(room).or_default().insert(user);
    }

    pub(super) fn is_user_banned(&self, room: RoomId, user: UserId) -> bool {
        self.banned_users
            .get(&room)
            .map(|b| b.contains(&user))
            .unwrap_or_default()
    }

    pub(super) fn delete_user_bans(&mut self, room: RoomId) {
        self.banned_users.remove(&room);
    }

    pub(super) fn init_waiting_room_enabled(&mut self, room: RoomId, enabled: bool) -> bool {
        *self.waiting_room_enabled.entry(room).or_insert(enabled)
    }

    pub(super) fn set_waiting_room_enabled(&mut self, room: RoomId, enabled: bool) {
        self.waiting_room_enabled.insert(room, enabled);
    }

    pub(super) fn is_waiting_room_enabled(&self, room: RoomId) -> bool {
        self.waiting_room_enabled
            .get(&room)
            .copied()
            .unwrap_or_default()
    }

    pub(super) fn delete_waiting_room_enabled(&mut self, room: RoomId) {
        self.waiting_room_enabled.remove(&room);
    }

    pub(super) fn set_raise_hands_enabled(&mut self, room: RoomId, enabled: bool) {
        self.raise_hands_enabled.insert(room, enabled);
    }

    pub(super) fn is_raise_hands_enabled(&self, room: RoomId) -> bool {
        self.raise_hands_enabled.get(&room).copied().unwrap_or(true)
    }

    pub(super) fn delete_raise_hands_enabled(&mut self, room: RoomId) {
        self.raise_hands_enabled.remove(&room);
    }

    pub(super) fn waiting_room_add_participant(
        &mut self,
        room: RoomId,
        participant: ParticipantId,
    ) -> bool {
        self.waiting_room_participants
            .entry(room)
            .or_default()
            .insert(participant)
    }

    pub(super) fn waiting_room_remove_participant(
        &mut self,
        room: RoomId,
        participant: ParticipantId,
    ) {
        self.waiting_room_participants
            .get_mut(&room)
            .map(|p| p.remove(&participant));
    }

    pub(super) fn waiting_room_contains_participant(
        &self,
        room: RoomId,
        participant: ParticipantId,
    ) -> bool {
        self.waiting_room_participants
            .get(&room)
            .map(|p| p.contains(&participant))
            .unwrap_or_default()
    }

    pub(super) fn waiting_room_participants(&self, room: RoomId) -> BTreeSet<ParticipantId> {
        BTreeSet::from_iter(
            self.waiting_room_participants
                .get(&room)
                .map(|p| p.iter())
                .into_iter()
                .flatten()
                .copied(),
        )
    }

    pub(super) fn waiting_room_participant_count(&self, room: RoomId) -> usize {
        self.waiting_room_participants
            .get(&room)
            .map(|p| p.len())
            .unwrap_or_default()
    }

    pub(super) fn delete_waiting_room(&mut self, room: RoomId) {
        self.waiting_room_participants.remove(&room);
    }

    pub(super) fn waiting_room_accepted_add_participant(
        &mut self,
        room: RoomId,
        participant: ParticipantId,
    ) -> bool {
        self.waiting_room_accepted_participants
            .entry(room)
            .or_default()
            .insert(participant)
    }

    pub(super) fn waiting_room_accepted_remove_participant(
        &mut self,
        room: RoomId,
        participant: ParticipantId,
    ) {
        self.waiting_room_accepted_participants
            .get_mut(&room)
            .map(|p| p.remove(&participant));
    }

    pub(super) fn waiting_room_accepted_remove_participants(
        &mut self,
        room: RoomId,
        participants: &[ParticipantId],
    ) {
        if let Some(p) = self.waiting_room_accepted_participants.get_mut(&room) {
            let to_be_removed = HashSet::from_iter(participants.iter().copied());
            *p = &*p - &to_be_removed;
        }
    }

    pub(super) fn waiting_room_accepted_participants(
        &self,
        room: RoomId,
    ) -> BTreeSet<ParticipantId> {
        BTreeSet::from_iter(
            self.waiting_room_accepted_participants
                .get(&room)
                .map(|p| p.iter())
                .into_iter()
                .flatten()
                .copied(),
        )
    }

    pub(super) fn waiting_room_accepted_participant_count(&self, room: RoomId) -> usize {
        self.waiting_room_accepted_participants
            .get(&room)
            .map(|p| p.len())
            .unwrap_or_default()
    }

    pub(super) fn delete_waiting_room_accepted(&mut self, room: RoomId) {
        self.waiting_room_accepted_participants.remove(&room);
    }
}
