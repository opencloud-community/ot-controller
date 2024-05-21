// SPDX-FileCopyrightText: OpenTalk GmbH <mail@opentalk.eu>
//
// SPDX-License-Identifier: EUPL-1.2

use opentalk_signaling_core::{ExpiringDataHashMap, SignalingRoomId};
use opentalk_types::signaling::polls::state::PollsState;

#[derive(Debug, Clone, Default)]
pub(super) struct MemoryPollsState {
    polls_state: ExpiringDataHashMap<SignalingRoomId, PollsState>,
}

impl MemoryPollsState {
    #[cfg(test)]
    pub(super) fn reset(&mut self) {
        *self = Self::default();
    }

    pub(super) fn get_polls_state(&self, room: SignalingRoomId) -> Option<PollsState> {
        self.polls_state.get(&room).cloned()
    }

    pub(super) fn set_polls_state(
        &mut self,
        room: SignalingRoomId,
        polls_state: &PollsState,
    ) -> bool {
        self.polls_state.insert_with_expiry_if_not_exists(
            room,
            polls_state.clone(),
            polls_state.duration,
        )
    }
}
