// SPDX-FileCopyrightText: OpenTalk GmbH <mail@opentalk.eu>
//
// SPDX-License-Identifier: EUPL-1.2

use std::collections::{HashMap, HashSet};

use opentalk_types::core::{RoomId, UserId};

#[derive(Debug, Clone, Default)]
pub(super) struct MemoryModerationState {
    banned_users: HashMap<RoomId, HashSet<UserId>>,
    waiting_room_enabled: HashMap<RoomId, bool>,
    raise_hands_enabled: HashMap<RoomId, bool>,
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
}
