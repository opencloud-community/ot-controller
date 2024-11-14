// SPDX-FileCopyrightText: OpenTalk GmbH <mail@opentalk.eu>
//
// SPDX-License-Identifier: EUPL-1.2

use std::collections::{BTreeSet, HashMap};

use opentalk_types_common::rooms::RoomId;
use opentalk_types_signaling::ParticipantId;
use opentalk_types_signaling_livekit::MicrophoneRestrictionState;

#[derive(Debug, Clone, Default)]
pub(super) struct MemoryLivekitState {
    force_mute: HashMap<RoomId, BTreeSet<ParticipantId>>,
}

impl MemoryLivekitState {
    #[cfg(test)]
    pub(super) fn reset(&mut self) {
        *self = Self::default();
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

    pub(super) fn get_force_mute_state(&self, room: RoomId) -> MicrophoneRestrictionState {
        match self.force_mute.get(&room).cloned() {
            None => MicrophoneRestrictionState::Disabled,
            Some(unrestricted_participants) => MicrophoneRestrictionState::Enabled {
                unrestricted_participants: unrestricted_participants.clone(),
            },
        }
    }
}
