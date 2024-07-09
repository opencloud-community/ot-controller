// SPDX-FileCopyrightText: OpenTalk GmbH <mail@opentalk.eu>
//
// SPDX-License-Identifier: EUPL-1.2

use std::collections::{BTreeMap, BTreeSet, HashMap, HashSet};

use opentalk_signaling_core::SignalingRoomId;
use opentalk_types::{
    core::{ParticipantId, RoomId, Timestamp},
    signaling::media::{
        state::ForceMuteState, ParticipantMediaState, ParticipantSpeakingState, SpeakingState,
    },
};

use crate::mcu::{McuId, MediaSessionKey, PublisherInfo};

#[derive(Debug, Clone, Default)]
pub(super) struct MemoryMediaState {
    participant_media_states: HashMap<(SignalingRoomId, ParticipantId), ParticipantMediaState>,
    presenters: HashMap<SignalingRoomId, HashSet<ParticipantId>>,
    speakers: HashMap<(SignalingRoomId, ParticipantId), SpeakingState>,
    mcu_load: HashMap<(McuId, Option<usize>), usize>,
    publisher_info: HashMap<MediaSessionKey, PublisherInfo>,
    force_mute: HashMap<RoomId, BTreeSet<ParticipantId>>,
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
        self.speakers.insert(
            (room, participant),
            SpeakingState {
                is_speaking,
                updated_at,
            },
        );
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

    pub(super) fn delete_speaking_state_multiple_participants(
        &mut self,
        room: SignalingRoomId,
        participants: &[ParticipantId],
    ) {
        for k in participants.iter().copied().map(|p| (room, p)) {
            self.speakers.remove(&k);
        }
    }

    pub(super) fn get_speaking_state_multiple_participants(
        &self,
        room: SignalingRoomId,
        participants: &[ParticipantId],
    ) -> Vec<ParticipantSpeakingState> {
        participants
            .iter()
            .copied()
            .filter_map(|participant| {
                self.get_speaking_state(room, participant)
                    .map(|state| ParticipantSpeakingState {
                        participant,
                        speaker: state,
                    })
            })
            .collect()
    }

    pub(super) fn initialize_mcu_load(&mut self, mcu_id: McuId, index: Option<usize>) {
        self.mcu_load.insert((mcu_id, index), 0);
    }

    pub(super) fn get_mcus_sorted_by_load(&self) -> Vec<(McuId, Option<usize>)> {
        let mut categories: BTreeMap<usize, BTreeSet<&(McuId, Option<usize>)>> = BTreeMap::new();

        for (id, load) in self.mcu_load.iter() {
            categories.entry(*load).or_default().insert(id);
        }

        Vec::from_iter(categories.into_values().flatten().cloned())
    }

    pub(super) fn increase_mcu_load(&mut self, mcu_id: McuId, index: Option<usize>) {
        let load = self.mcu_load.entry((mcu_id, index)).or_default();
        *load = load.saturating_add(1);
    }

    pub(super) fn decrease_mcu_load(&mut self, mcu_id: McuId, index: Option<usize>) {
        let load = self.mcu_load.entry((mcu_id, index)).or_default();
        *load = load.saturating_sub(1);
    }

    pub(super) fn set_publisher_info(
        &mut self,
        media_session_key: MediaSessionKey,
        info: PublisherInfo,
    ) {
        self.publisher_info.insert(media_session_key, info);
    }

    pub(super) fn get_publisher_info(
        &self,
        media_session_key: MediaSessionKey,
    ) -> Option<PublisherInfo> {
        self.publisher_info.get(&media_session_key).cloned()
    }

    pub(super) fn delete_publisher_info(&mut self, media_session_key: MediaSessionKey) {
        self.publisher_info.remove(&media_session_key);
    }

    pub(super) fn force_mute_set_allow_unmute(
        &mut self,
        room: RoomId,
        participants: &[ParticipantId],
    ) {
        if participants.is_empty() {
            self.clear_force_mute(room);
            return;
        }
        self.force_mute
            .insert(room, BTreeSet::from_iter(participants.iter().copied()));
    }

    pub(super) fn clear_force_mute(&mut self, room: RoomId) {
        self.force_mute.remove(&room);
    }

    pub(super) fn get_force_mute_state(&self, room: RoomId) -> ForceMuteState {
        match self.force_mute.get(&room).cloned() {
            None => ForceMuteState::Disabled,
            Some(allow_list) => ForceMuteState::Enabled {
                allow_list: allow_list.clone(),
            },
        }
    }

    pub(super) fn is_unmute_allowed(&self, room: RoomId, participant: ParticipantId) -> bool {
        match self.force_mute.get(&room) {
            None => true,
            Some(allow_list) => allow_list.contains(&participant),
        }
    }
}
