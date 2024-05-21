// SPDX-FileCopyrightText: OpenTalk GmbH <mail@opentalk.eu>
//
// SPDX-License-Identifier: EUPL-1.2

use std::collections::{hash_map::Entry, HashMap};

use opentalk_signaling_core::SignalingRoomId;

use crate::storage::redis::InitState;

#[derive(Debug, Clone, Default)]
pub(crate) struct MemoryProtocolState {
    group_ids: HashMap<SignalingRoomId, String>,
    init_state: HashMap<SignalingRoomId, InitState>,
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

    pub(crate) fn init_get_or_default(&mut self, room: SignalingRoomId) -> Option<InitState> {
        match self.init_state.entry(room) {
            Entry::Occupied(occupied) => Some(*occupied.get()),
            Entry::Vacant(vacant) => {
                vacant.insert(InitState::Initializing);
                None
            }
        }
    }
}
