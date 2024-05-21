// SPDX-FileCopyrightText: OpenTalk GmbH <mail@opentalk.eu>
//
// SPDX-License-Identifier: EUPL-1.2

use std::collections::{BTreeMap, BTreeSet};

use async_trait::async_trait;
use opentalk_signaling_core::{SignalingModuleError, SignalingRoomId};
use opentalk_types::signaling::polls::{state::PollsState, ChoiceId, PollId};

#[async_trait(?Send)]
pub(crate) trait PollsStorage {
    async fn get_polls_state(
        &mut self,
        room: SignalingRoomId,
    ) -> Result<Option<PollsState>, SignalingModuleError>;

    /// Set the current polls state if one doesn't already exist returns true if set was successful
    async fn set_polls_state(
        &mut self,
        room: SignalingRoomId,
        polls_state: &PollsState,
    ) -> Result<bool, SignalingModuleError>;

    async fn delete_polls_state(
        &mut self,
        room: SignalingRoomId,
    ) -> Result<(), SignalingModuleError>;

    async fn delete_poll_results(
        &mut self,
        room: SignalingRoomId,
        poll_id: PollId,
    ) -> Result<(), SignalingModuleError>;

    async fn add_poll_to_list(
        &mut self,
        room: SignalingRoomId,
        poll_id: PollId,
    ) -> Result<(), SignalingModuleError>;

    async fn results(
        &mut self,
        room: SignalingRoomId,
        poll: PollId,
    ) -> Result<BTreeMap<ChoiceId, u32>, SignalingModuleError>;

    async fn vote(
        &mut self,
        room: SignalingRoomId,
        poll_id: PollId,
        previous_choice_ids: &BTreeSet<ChoiceId>,
        new_choice_ids: &BTreeSet<ChoiceId>,
    ) -> Result<(), SignalingModuleError>;

    /// Get all polls for the room
    async fn poll_ids(
        &mut self,
        room: SignalingRoomId,
    ) -> Result<Vec<PollId>, SignalingModuleError>;
}
