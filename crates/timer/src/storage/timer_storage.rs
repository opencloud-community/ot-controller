// SPDX-FileCopyrightText: OpenTalk GmbH <mail@opentalk.eu>
//
// SPDX-License-Identifier: EUPL-1.2

use async_trait::async_trait;
use opentalk_signaling_core::{SignalingModuleError, SignalingRoomId};
use opentalk_types::core::ParticipantId;

#[async_trait(?Send)]
pub(crate) trait TimerStorage {
    /// Set the ready status of a participant
    async fn ready_status_set(
        &mut self,
        room_id: SignalingRoomId,
        participant_id: ParticipantId,
        ready_status: bool,
    ) -> Result<(), SignalingModuleError>;
}
