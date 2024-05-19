// SPDX-FileCopyrightText: OpenTalk GmbH <mail@opentalk.eu>
//
// SPDX-License-Identifier: EUPL-1.2

use std::collections::{HashMap, HashSet};

use opentalk_signaling_core::SignalingRoomId;
use opentalk_types::{
    core::{ParticipantId, Timestamp},
    signaling::media::{ParticipantMediaState, SpeakingState},
};

#[derive(Debug, Clone, Default)]
pub(super) struct MemoryMediaState {
    participant_media_states: HashMap<(SignalingRoomId, ParticipantId), ParticipantMediaState>,
    presenters: HashMap<SignalingRoomId, HashSet<ParticipantId>>,
    speakers: HashMap<(SignalingRoomId, ParticipantId), SpeakingState>,
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

    pub(super) fn add_presenter(&mut self, room: SignalingRoomId, participant: ParticipantId) {
        self.presenters.entry(room).or_default().insert(participant);
    }

    pub(super) fn is_presenter(&self, room: SignalingRoomId, participant: ParticipantId) -> bool {
        self.presenters
            .get(&room)
            .map(|p| p.contains(&participant))
            .unwrap_or_default()
    }

    pub(super) fn remove_presenter(&mut self, room: SignalingRoomId, participant: ParticipantId) {
        self.presenters
            .get_mut(&room)
            .map(|p| p.remove(&participant));
    }

    pub(super) fn clear_presenters(&mut self, room: SignalingRoomId) {
        self.presenters.remove(&room);
    }

    pub(super) fn set_speaking_state(
        &mut self,
        room: SignalingRoomId,
        participant: ParticipantId,
        is_speaking: bool,
        updated_at: Timestamp,
    ) {
        self.speakers
            .entry((room, participant))
            .or_insert_with(|| SpeakingState {
                is_speaking,
                updated_at,
            });
    }

    pub(super) fn get_speaking_state(
        &self,
        room: SignalingRoomId,
        participant: ParticipantId,
    ) -> Option<SpeakingState> {
        self.speakers.get(&(room, participant)).cloned()
    }

    pub(super) fn delete_speaking_state(
        &mut self,
        room: SignalingRoomId,
        participant: ParticipantId,
    ) {
        self.speakers.remove(&(room, participant));
    }
}
