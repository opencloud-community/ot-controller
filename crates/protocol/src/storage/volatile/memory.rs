// SPDX-FileCopyrightText: OpenTalk GmbH <mail@opentalk.eu>
//
// SPDX-License-Identifier: EUPL-1.2

use std::collections::HashMap;

use opentalk_signaling_core::SignalingRoomId;

#[derive(Debug, Clone, Default)]
pub(crate) struct MemoryProtocolState {
    group_ids: HashMap<SignalingRoomId, String>,
}

impl MemoryProtocolState {
    #[cfg(test)]
    pub(super) fn reset(&mut self) {
        *self = Self::default();
    }

    pub(crate) fn group_set(&mut self, room: SignalingRoomId, group: &str) {
        self.group_ids.insert(room, group.to_string());
    }

    pub(crate) fn group_get(&self, room: SignalingRoomId) -> Option<String> {
        self.group_ids.get(&room).cloned()
    }

    pub(crate) fn group_delete(&mut self, room: SignalingRoomId) -> Option<String> {
        self.group_ids.remove(&room)
    }
}
