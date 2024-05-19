// SPDX-FileCopyrightText: OpenTalk GmbH <mail@opentalk.eu>
//
// SPDX-License-Identifier: EUPL-1.2

use std::collections::HashMap;

use opentalk_signaling_core::SignalingRoomId;
use opentalk_types::{core::ParticipantId, signaling::media::ParticipantMediaState};

#[derive(Debug, Clone, Default)]
pub(super) struct MemoryMediaState {
    participant_media_states: HashMap<(SignalingRoomId, ParticipantId), ParticipantMediaState>,
}

impl MemoryMediaState {
    #[cfg(test)]
    pub(super) fn reset(&mut self) {
        *self = Self::default();
    }

    pub(super) fn get_media_state(
        &self,
        room: SignalingRoomId,
        participant: ParticipantId,
    ) -> Option<ParticipantMediaState> {
        self.participant_media_states
            .get(&(room, participant))
            .cloned()
    }
}
