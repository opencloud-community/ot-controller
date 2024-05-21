// SPDX-FileCopyrightText: OpenTalk GmbH <mail@opentalk.eu>
//
// SPDX-License-Identifier: EUPL-1.2

use std::collections::HashMap;

use opentalk_signaling_core::{ExpiringDataHashMap, SignalingRoomId};
use opentalk_types::signaling::polls::{state::PollsState, ChoiceId, PollId};

#[derive(Debug, Clone, Default)]
pub(super) struct MemoryPollsState {
    polls_state: ExpiringDataHashMap<SignalingRoomId, PollsState>,
    poll_results: HashMap<(SignalingRoomId, PollId), HashMap<ChoiceId, u32>>,
    poll_ids: HashMap<SignalingRoomId, Vec<PollId>>,
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

    pub(super) fn delete_polls_state(&mut self, room: &SignalingRoomId) -> Option<PollsState> {
        self.polls_state.remove(room)
    }

    pub(super) fn delete_polls_results(&mut self, room: SignalingRoomId, poll: PollId) {
        self.poll_results.remove(&(room, poll));
    }

    pub(super) fn add_poll_to_list(&mut self, room: SignalingRoomId, poll_id: PollId) {
        let set = self.poll_ids.entry(room).or_default();
        set.push(poll_id);
    }
}
