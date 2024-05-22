// SPDX-FileCopyrightText: OpenTalk GmbH <mail@opentalk.eu>
//
// SPDX-License-Identifier: EUPL-1.2

use std::collections::BTreeMap;

use opentalk_signaling_core::SignalingRoomId;
use opentalk_types::{core::ParticipantId, signaling::timer::ready_status::ReadyStatus};

#[derive(Debug, Clone, Default)]
pub(crate) struct MemoryTimerState {
    ready_status: BTreeMap<(SignalingRoomId, ParticipantId), bool>,
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
        self.ready_status
            .insert((room_id, participant_id), ready_status);
    }

    pub(super) fn ready_status_get(
        &self,
        room: SignalingRoomId,
        participant: ParticipantId,
    ) -> Option<ReadyStatus> {
        self.ready_status
            .get(&(room, participant))
            .map(|&ready_status| ReadyStatus { ready_status })
    }
}
