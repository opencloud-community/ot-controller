// SPDX-FileCopyrightText: OpenTalk GmbH <mail@opentalk.eu>
//
// SPDX-License-Identifier: EUPL-1.2

use std::collections::{HashMap, HashSet};

use opentalk_types::core::{RoomId, UserId};

#[derive(Debug, Clone, Default)]
pub(super) struct MemoryModerationState {
    banned_users: HashMap<RoomId, HashSet<UserId>>,
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
}
