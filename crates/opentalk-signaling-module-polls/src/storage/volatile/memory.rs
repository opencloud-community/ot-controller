// SPDX-FileCopyrightText: OpenTalk GmbH <mail@opentalk.eu>
//
// SPDX-License-Identifier: EUPL-1.2

use std::collections::{BTreeMap, BTreeSet, HashMap};

use opentalk_signaling_core::{ExpiringDataHashMap, SignalingModuleError, SignalingRoomId};
use opentalk_types_signaling_polls::{state::PollsState, ChoiceId, PollId};
use snafu::OptionExt;

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

    pub(super) fn poll_ids(&self, room: SignalingRoomId) -> Vec<PollId> {
        self.poll_ids.get(&room).cloned().unwrap_or_default()
    }

    pub(super) fn delete_poll_ids(&mut self, room: SignalingRoomId) {
        self.poll_ids.remove(&room);
    }

    pub(super) fn poll_results(
        &self,
        room: SignalingRoomId,
        poll: PollId,
    ) -> Option<BTreeMap<ChoiceId, u32>> {
        self.poll_results
            .get(&(room, poll))
            .cloned()
            .map(BTreeMap::from_iter)
    }

    pub(super) fn vote(
        &mut self,
        room: SignalingRoomId,
        poll: PollId,
        previous_choices: &BTreeSet<ChoiceId>,
        new_choices: &BTreeSet<ChoiceId>,
    ) -> Result<(), SignalingModuleError> {
        // Clone first to ensure the no changes are applied if we encounter errors
        let mut choices = self
            .poll_results
            .get(&(room, poll))
            .cloned()
            .unwrap_or_default();
        for removed_choice in previous_choices.difference(new_choices) {
            let counter = choices
                .get_mut(removed_choice)
                .whatever_context::<_, SignalingModuleError>("Choice not found")?;
            *counter -= 1;
        }

        for added_choice in new_choices.difference(previous_choices) {
            let counter = choices.entry(*added_choice).or_default();
            *counter += 1;
        }

        self.poll_results.insert((room, poll), choices);
        Ok(())
    }
}
