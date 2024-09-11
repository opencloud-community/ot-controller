// SPDX-FileCopyrightText: OpenTalk GmbH <mail@opentalk.eu>
//
// SPDX-License-Identifier: EUPL-1.2

use async_trait::async_trait;
use opentalk_signaling_core::{SignalingModuleError, SignalingRoomId};
use opentalk_types::signaling::timer::ready_status::ReadyStatus;
use opentalk_types_signaling::ParticipantId;

use super::Timer;

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

    /// Attempt to set a new timer
    ///
    /// Returns `true` when the new timer was created
    /// Returns `false` when a timer is already active
    async fn timer_set_if_not_exists(
        &mut self,
        room_id: SignalingRoomId,
        timer: &Timer,
    ) -> Result<bool, SignalingModuleError>;

    /// Get the current meeting timer
    async fn timer_get(
        &mut self,
        room_id: SignalingRoomId,
    ) -> Result<Option<Timer>, SignalingModuleError>;

    /// Delete the current timer
    ///
    /// Returns the timer if there was any
    async fn timer_delete(
        &mut self,
        room_id: SignalingRoomId,
    ) -> Result<Option<Timer>, SignalingModuleError>;
}
