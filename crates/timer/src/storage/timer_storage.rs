// SPDX-FileCopyrightText: OpenTalk GmbH <mail@opentalk.eu>
//
// SPDX-License-Identifier: EUPL-1.2

use async_trait::async_trait;
use opentalk_signaling_core::{SignalingModuleError, SignalingRoomId};
use opentalk_types::{core::ParticipantId, signaling::timer::ready_status::ReadyStatus};

#[async_trait(?Send)]
pub(crate) trait TimerStorage {
    /// Set the ready status of a participant
    async fn ready_status_set(
        &mut self,
        room_id: SignalingRoomId,
        participant_id: ParticipantId,
        ready_status: bool,
    ) -> Result<(), SignalingModuleError>;

    /// Get the ready status of a participant
    async fn ready_status_get(
        &mut self,
        room_id: SignalingRoomId,
        participant_id: ParticipantId,
    ) -> Result<Option<ReadyStatus>, SignalingModuleError>;

    /// Delete the ready status of a participant
    async fn ready_status_delete(
        &mut self,
        room_id: SignalingRoomId,
        participant_id: ParticipantId,
    ) -> Result<(), SignalingModuleError>;
}
