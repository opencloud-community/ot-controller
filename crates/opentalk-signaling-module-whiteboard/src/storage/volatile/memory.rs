// SPDX-FileCopyrightText: OpenTalk GmbH <mail@opentalk.eu>
//
// SPDX-License-Identifier: EUPL-1.2

use std::collections::{BTreeMap, btree_map::Entry};

use opentalk_signaling_core::SignalingRoomId;

use crate::storage::{InitState, SpaceInfo};

#[derive(Debug, Clone, Default)]
pub(crate) struct MemoryWhiteboardState {
    init_state: BTreeMap<SignalingRoomId, InitState>,
}

impl MemoryWhiteboardState {
    #[cfg(test)]
    pub(super) fn reset(&mut self) {
        *self = Self::default();
    }

    pub(super) fn init_get_or_default(&mut self, room: SignalingRoomId) -> Option<InitState> {
        match self.init_state.entry(room) {
            Entry::Occupied(occupied) => Some(occupied.get().clone()),
            Entry::Vacant(vacant) => {
                vacant.insert(InitState::Initializing);
                None
            }
        }
    }

    pub(super) fn set_initialized(&mut self, room: SignalingRoomId, space_info: SpaceInfo) {
        self.init_state
            .insert(room, InitState::Initialized(space_info));
    }

    pub(super) fn get_init_state(&self, room: SignalingRoomId) -> Option<InitState> {
        self.init_state.get(&room).cloned()
    }

    pub(super) fn delete_init_state(&mut self, room: SignalingRoomId) {
        self.init_state.remove(&room);
    }
}
