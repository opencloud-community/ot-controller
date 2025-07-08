// SPDX-FileCopyrightText: OpenTalk GmbH <mail@opentalk.eu>
//
// SPDX-License-Identifier: EUPL-1.2

use std::collections::{BTreeMap, HashMap, hash_map::Entry};

use opentalk_signaling_core::{NotFoundSnafu, SignalingModuleError, SignalingRoomId};
use opentalk_types_signaling::ParticipantId;
use opentalk_types_signaling_subroom_audio::{state::WhisperState, whisper_id::WhisperId};
use snafu::{OptionExt, whatever};

#[derive(Default)]
pub(crate) struct MemorySubroomAudio {
    state: HashMap<SignalingRoomId, HashMap<WhisperId, BTreeMap<ParticipantId, WhisperState>>>,
}

impl MemorySubroomAudio {
    #[cfg(test)]
    pub(crate) fn reset(&mut self) {
        *self = Self::default();
    }

    pub(crate) fn create_whisper_group(
        &mut self,
        room_id: SignalingRoomId,
        whisper_id: WhisperId,
        participants: &BTreeMap<ParticipantId, WhisperState>,
    ) -> Result<(), SignalingModuleError> {
        let whisper_group_entry = self.state.entry(room_id).or_default().entry(whisper_id);

        match whisper_group_entry {
            Entry::Vacant(vacant_entry) => {
                vacant_entry.insert(participants.clone());
            }
            Entry::Occupied(_) => {
                whatever!("Whisper group already exists")
            }
        }

        Ok(())
    }

    pub(crate) fn get_whisper_group(
        &self,
        room_id: SignalingRoomId,
        whisper_id: WhisperId,
    ) -> Result<BTreeMap<ParticipantId, WhisperState>, SignalingModuleError> {
        let whisper_groups = match self.state.get(&room_id) {
            Some(groups) => groups,
            None => return Ok(BTreeMap::default()),
        };

        let participants = whisper_groups.get(&whisper_id).cloned().unwrap_or_default();

        Ok(participants)
    }

    pub(crate) fn get_whisper_group_ids(&self, room: SignalingRoomId) -> Vec<WhisperId> {
        let Some(whisper_groups) = self.state.get(&room) else {
            return Vec::default();
        };

        whisper_groups.keys().copied().collect()
    }

    pub(crate) fn remove_whisper_group(&mut self, room: SignalingRoomId, whisper_id: WhisperId) {
        let Some(whisper_groups) = self.state.get_mut(&room) else {
            return;
        };

        whisper_groups.remove(&whisper_id);
    }

    pub(crate) fn add_participants(
        &mut self,
        room: SignalingRoomId,
        whisper_id: WhisperId,
        participant_ids: &BTreeMap<ParticipantId, WhisperState>,
    ) {
        let whisper_group = self
            .state
            .entry(room)
            .or_default()
            .entry(whisper_id)
            .or_default();

        whisper_group.extend(participant_ids.iter());
    }

    pub(crate) fn remove_participant(
        &mut self,
        room: SignalingRoomId,
        whisper_id: WhisperId,
        participant_id: ParticipantId,
    ) -> Result<bool, SignalingModuleError> {
        let whisper_groups = self.state.entry(room).or_default();

        let whisper_participants = whisper_groups.get_mut(&whisper_id).context(NotFoundSnafu {
            message: "Whisper id does not exist",
        })?;

        whisper_participants
            .remove(&participant_id)
            .context(NotFoundSnafu {
                message: "participant id does not exist in whisper group",
            })?;

        if whisper_participants.is_empty() {
            whisper_groups.remove(&whisper_id);
            Ok(true)
        } else {
            Ok(false)
        }
    }

    pub(crate) fn update_participant_state(
        &mut self,
        room: SignalingRoomId,
        whisper_id: WhisperId,
        participant_id: ParticipantId,
        participant_state: WhisperState,
    ) -> Result<(), SignalingModuleError> {
        let state = self
            .state
            .entry(room)
            .or_default()
            .get_mut(&whisper_id)
            .context(NotFoundSnafu {
                message: "Whisper id does not exist",
            })?
            .get_mut(&participant_id)
            .context(NotFoundSnafu {
                message: "participant id does not exist in whisper group",
            })?;

        *state = participant_state;

        Ok(())
    }
}
