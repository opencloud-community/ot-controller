// SPDX-FileCopyrightText: OpenTalk GmbH <mail@opentalk.eu>
//
// SPDX-License-Identifier: EUPL-1.2

use std::collections::{HashMap, HashSet};

use opentalk_signaling_core::SignalingRoomId;
use opentalk_types::{core::ParticipantId, signaling::media::ParticipantMediaState};

#[derive(Debug, Clone, Default)]
pub(super) struct MemoryMediaState {
    participant_media_states: HashMap<(SignalingRoomId, ParticipantId), ParticipantMediaState>,
    presenters: HashMap<SignalingRoomId, HashSet<ParticipantId>>,
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

    pub(super) fn set_media_state(
        &mut self,
        room: SignalingRoomId,
        participant: ParticipantId,
        participant_media_state: &ParticipantMediaState,
    ) {
        self.participant_media_states
            .insert((room, participant), participant_media_state.clone());
    }

    pub(super) fn delete_media_state(&mut self, room: SignalingRoomId, participant: ParticipantId) {
        self.participant_media_states.remove(&(room, participant));
    }

    pub(super) fn set_is_presenter(&mut self, room: SignalingRoomId, participant: ParticipantId) {
        self.presenters.entry(room).or_default().insert(participant);
    }

    #[tracing::instrument(level = "debug", skip(self))]
    pub(super) fn is_presenter(&self, room: SignalingRoomId, participant: ParticipantId) -> bool {
        self.presenters
            .get(&room)
            .map(|p| p.contains(&participant))
            .unwrap_or_default()
    }
}
