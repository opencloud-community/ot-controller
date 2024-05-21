// SPDX-FileCopyrightText: OpenTalk GmbH <mail@opentalk.eu>
//
// SPDX-License-Identifier: EUPL-1.2

use std::collections::HashMap;

use opentalk_signaling_core::SignalingRoomId;
use opentalk_types::signaling::polls::state::PollsState;

#[derive(Debug, Clone, Default)]
pub(super) struct MemoryPollsState {
    polls_state: HashMap<SignalingRoomId, PollsState>,
}

impl MemoryPollsState {
    pub(super) fn get_polls_state(&self, room: SignalingRoomId) -> Option<PollsState> {
        self.polls_state.get(&room).cloned()
    }
}
