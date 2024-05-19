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
}
