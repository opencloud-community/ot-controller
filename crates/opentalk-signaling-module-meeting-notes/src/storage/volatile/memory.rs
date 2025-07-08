// SPDX-FileCopyrightText: OpenTalk GmbH <mail@opentalk.eu>
//
// SPDX-License-Identifier: EUPL-1.2

use std::collections::{HashMap, hash_map::Entry};

use opentalk_signaling_core::SignalingRoomId;
use opentalk_types_signaling::ParticipantId;

use crate::{SessionInfo, storage::InitState};

#[derive(Debug, Clone, Default)]
pub(crate) struct MemoryMeetingNotesState {
    group_ids: HashMap<SignalingRoomId, String>,
    init_state: HashMap<SignalingRoomId, InitState>,
    session: HashMap<(SignalingRoomId, ParticipantId), SessionInfo>,
}

impl MemoryMeetingNotesState {
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

    pub(crate) fn set_initialized(&mut self, room: SignalingRoomId) {
        self.init_state.insert(room, InitState::Initialized);
    }

    pub(crate) fn init_get(&self, room: SignalingRoomId) -> Option<InitState> {
        self.init_state.get(&room).copied()
    }

    pub(crate) fn init_delete(&mut self, room: SignalingRoomId) {
        self.init_state.remove(&room);
    }

    pub(crate) fn session_get(
        &self,
        room: SignalingRoomId,
        participant: ParticipantId,
    ) -> Option<SessionInfo> {
        self.session.get(&(room, participant)).cloned()
    }

    pub(crate) fn session_set(
        &mut self,
        room: SignalingRoomId,
        participant: ParticipantId,
        session_info: SessionInfo,
    ) {
        self.session.insert((room, participant), session_info);
    }

    pub(crate) fn session_delete(
        &mut self,
        room: SignalingRoomId,
        participant: ParticipantId,
    ) -> Option<SessionInfo> {
        self.session.remove(&(room, participant))
    }
}
