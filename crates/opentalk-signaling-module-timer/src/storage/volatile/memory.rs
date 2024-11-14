// SPDX-FileCopyrightText: OpenTalk GmbH <mail@opentalk.eu>
//
// SPDX-License-Identifier: EUPL-1.2

use std::collections::BTreeMap;

use opentalk_signaling_core::SignalingRoomId;
use opentalk_types_signaling::ParticipantId;
use opentalk_types_signaling_timer::peer_state::TimerPeerState;

use crate::storage::Timer;

#[derive(Debug, Clone, Default)]
pub(crate) struct MemoryTimerState {
    ready_status: BTreeMap<(SignalingRoomId, ParticipantId), bool>,
    timers: BTreeMap<SignalingRoomId, Timer>,
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
    ) -> Option<TimerPeerState> {
        self.ready_status
            .get(&(room, participant))
            .map(|&ready_status| TimerPeerState { ready_status })
    }

    pub(super) fn ready_status_delete(
        &mut self,
        room: SignalingRoomId,
        participant: ParticipantId,
    ) {
        self.ready_status.remove(&(room, participant));
    }

    pub(super) fn timer_set_if_not_exists(&mut self, room: SignalingRoomId, timer: Timer) -> bool {
        let already_set = self.timers.contains_key(&room);
        self.timers.entry(room).or_insert(timer);
        !already_set
    }

    pub(super) fn timer_get(&self, room: SignalingRoomId) -> Option<Timer> {
        self.timers.get(&room).cloned()
    }

    pub(super) fn timer_delete(&mut self, room: SignalingRoomId) -> Option<Timer> {
        self.timers.remove(&room)
    }
}
