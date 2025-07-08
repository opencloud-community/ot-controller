// SPDX-FileCopyrightText: OpenTalk GmbH <mail@opentalk.eu>
//
// SPDX-License-Identifier: EUPL-1.2

use std::collections::{BTreeMap, BTreeSet};

use opentalk_signaling_core::{NotFoundSnafu, SignalingModuleError};
use opentalk_types_common::{
    rooms::RoomId,
    time::Timestamp,
    training_participation_report::{TimeRange, TrainingParticipationReportParameterSet},
};
use opentalk_types_signaling::ParticipantId;
use opentalk_types_signaling_training_participation_report::state::ParticipationLoggingState;
use snafu::{OptionExt as _, ensure_whatever};

use crate::storage::{Checkpoint, RoomState, TrainingReportState};

#[derive(Debug, Clone, Default, PartialEq, Eq)]
pub(crate) struct TrainingParticipationReportState {
    room_state: BTreeMap<RoomId, RoomState>,
    initialized: BTreeSet<RoomId>,
    parameter_sets: BTreeMap<RoomId, TrainingParticipationReportParameterSet>,
}

impl TrainingParticipationReportState {
    #[cfg(test)]
    pub(super) fn reset(&mut self) {
        *self = Self::default();
    }

    fn room(&self, room: RoomId) -> Result<&RoomState, SignalingModuleError> {
        self.room_state.get(&room).with_context(|| NotFoundSnafu {
            message: format!("failed to find training report state of room {room} for reading"),
        })
    }

    fn room_mut(&mut self, room: RoomId) -> Result<&mut RoomState, SignalingModuleError> {
        self.room_state
            .get_mut(&room)
            .with_context(|| NotFoundSnafu {
                message: format!("failed to find training report state of room {room} for writing"),
            })
    }

    pub(super) fn set_parameter_set_initialized(&mut self, room: RoomId) {
        _ = self.initialized.insert(room);
    }

    pub(super) fn get_parameter_set_initialized(&self, room: RoomId) -> bool {
        self.initialized.contains(&room)
    }

    pub(super) fn delete_parameter_set_initialized(&mut self, room: RoomId) {
        _ = self.initialized.remove(&room);
    }

    pub(super) fn get_parameter_set(
        &self,
        room: RoomId,
    ) -> Option<TrainingParticipationReportParameterSet> {
        self.parameter_sets.get(&room).cloned()
    }

    pub(super) fn set_parameter_set(
        &mut self,
        room: RoomId,
        value: TrainingParticipationReportParameterSet,
    ) {
        _ = self.parameter_sets.insert(room, value);
    }

    pub(super) fn delete_parameter_set(&mut self, room: RoomId) {
        _ = self.parameter_sets.remove(&room);
    }

    pub(super) fn initialize_room(
        &mut self,
        room: RoomId,
        start: Timestamp,
        report_state: TrainingReportState,
        initial_checkpoint_delay: TimeRange,
        checkpoint_interval: TimeRange,
        known_participants: BTreeSet<ParticipantId>,
    ) {
        _ = self.room_state.insert(
            room,
            RoomState {
                start,
                report_state,
                initial_checkpoint_delay,
                checkpoint_interval,
                history: vec![],
                next_checkpoint: None,
                known_participants,
            },
        );
    }

    pub(super) fn cleanup_room(&mut self, room: RoomId) -> Option<RoomState> {
        self.room_state.remove(&room)
    }

    pub(super) fn get_training_report_state(
        &self,
        room: RoomId,
    ) -> Result<Option<TrainingReportState>, SignalingModuleError> {
        Ok(self.room_state.get(&room).map(|r| r.report_state))
    }

    pub(super) fn set_training_report_state(
        &mut self,
        room: RoomId,
        report_state: TrainingReportState,
    ) -> Result<(), SignalingModuleError> {
        self.room_mut(room)?.report_state = report_state;
        Ok(())
    }

    pub(super) fn get_initial_checkpoint_delay(
        &self,
        room: RoomId,
    ) -> Result<TimeRange, SignalingModuleError> {
        Ok(self.room(room)?.initial_checkpoint_delay.clone())
    }

    pub(super) fn get_checkpoint_interval(
        &self,
        room: RoomId,
    ) -> Result<TimeRange, SignalingModuleError> {
        Ok(self.room(room)?.checkpoint_interval.clone())
    }

    pub(super) fn get_next_checkpoint(
        &self,
        room: RoomId,
    ) -> Result<Option<Timestamp>, SignalingModuleError> {
        Ok(self.room(room)?.next_checkpoint)
    }

    pub(super) fn add_known_participant(
        &mut self,
        room: RoomId,
        participant: ParticipantId,
    ) -> Result<(), SignalingModuleError> {
        _ = self.room_mut(room)?.known_participants.insert(participant);
        Ok(())
    }

    pub(super) fn switch_to_next_checkpoint(
        &mut self,
        room: RoomId,
        new_next_checkpoint: Timestamp,
    ) -> Result<(), SignalingModuleError> {
        let room_state = self.room_mut(room)?;

        let Some(next_checkpoint_timestamp) =
            room_state.next_checkpoint.replace(new_next_checkpoint)
        else {
            return Ok(());
        };

        let next_checkpoint = Checkpoint {
            timestamp: next_checkpoint_timestamp,
            presence: BTreeMap::new(),
        };
        room_state.history.push(next_checkpoint);
        Ok(())
    }

    pub(super) fn record_presence_confirmation(
        &mut self,
        room: RoomId,
        participant: ParticipantId,
        timestamp: Timestamp,
    ) -> Result<(), SignalingModuleError> {
        let room_state = self.room_mut(room)?;
        ensure_whatever!(
            room_state.report_state == TrainingReportState::TrackingPresence,
            "Cannot record presence confirmation when not in TrackingPresence state"
        );
        let current_checkpoint = room_state.history.last_mut()
            .with_whatever_context::<_, _, SignalingModuleError>(|| {
                format!("Cannot record presence confirmation for room {room} because it has no current checkpoint set")
            })?;
        _ = current_checkpoint.presence.insert(participant, timestamp);

        Ok(())
    }

    pub(super) fn get_recorded_presence_state(
        &self,
        room: RoomId,
        participant: ParticipantId,
    ) -> Result<ParticipationLoggingState, SignalingModuleError> {
        let Some(room_state) = self.room_state.get(&room) else {
            return Ok(ParticipationLoggingState::Disabled);
        };
        let Some(current_checkpoint) = room_state.history.last() else {
            return Ok(ParticipationLoggingState::Enabled);
        };
        if current_checkpoint.presence.contains_key(&participant) {
            Ok(ParticipationLoggingState::Enabled)
        } else {
            Ok(ParticipationLoggingState::WaitingForConfirmation)
        }
    }
}
