// SPDX-FileCopyrightText: OpenTalk GmbH <mail@opentalk.eu>
//
// SPDX-License-Identifier: EUPL-1.2

use async_trait::async_trait;
use opentalk_signaling_core::{SignalingModuleError, SignalingRoomId};
use opentalk_types::signaling::polls::state::PollsState;

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
}
