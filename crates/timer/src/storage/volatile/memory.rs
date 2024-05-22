// SPDX-FileCopyrightText: OpenTalk GmbH <mail@opentalk.eu>
//
// SPDX-License-Identifier: EUPL-1.2

use std::collections::BTreeSet;

use opentalk_signaling_core::SignalingRoomId;
use opentalk_types::core::ParticipantId;

#[derive(Debug, Clone, Default)]
pub(crate) struct MemoryTimerState {
    ready_status: BTreeSet<(SignalingRoomId, ParticipantId)>,
}

impl MemoryTimerState {
    #[cfg(test)]
    pub(super) fn reset(&mut self) {
        *self = Self::default();
    }

    pub(super) fn ready_status_set(
        &mut self,
        room_id: SignalingRoomId,
        participant_id: ParticipantId,
        ready_status: bool,
    ) {
        if ready_status {
            self.ready_status.insert((room_id, participant_id));
        } else {
            self.ready_status.remove(&(room_id, participant_id));
        }
    }
}
