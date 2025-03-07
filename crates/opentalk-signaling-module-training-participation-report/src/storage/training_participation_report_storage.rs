// SPDX-FileCopyrightText: OpenTalk GmbH <mail@opentalk.eu>
//
// SPDX-License-Identifier: EUPL-1.2

use std::collections::BTreeSet;

use async_trait::async_trait;
use opentalk_signaling_core::SignalingModuleError;
use opentalk_types_common::{
    rooms::RoomId, time::Timestamp, training_participation_report::TimeRange,
};
use opentalk_types_signaling::ParticipantId;
use opentalk_types_signaling_training_participation_report::state::ParticipationLoggingState;

use super::{RoomState, TrainingReportState};

#[async_trait(?Send)]
pub(crate) trait TrainingParticipationReportStorage {
    async fn initialize_room(
        &mut self,
        room: RoomId,
        start: Timestamp,
        report_state: TrainingReportState,
        initial_checkpoint_delay: TimeRange,
        checkpoint_interval: TimeRange,
        known_participants: BTreeSet<ParticipantId>,
    ) -> Result<(), SignalingModuleError>;

    async fn cleanup_room(
        &mut self,
        room: RoomId,
    ) -> Result<Option<RoomState>, SignalingModuleError>;

    async fn get_training_report_state(
        &mut self,
        room: RoomId,
    ) -> Result<Option<TrainingReportState>, SignalingModuleError>;

    async fn set_training_report_state(
        &mut self,
        room: RoomId,
        report_state: TrainingReportState,
    ) -> Result<(), SignalingModuleError>;

    async fn get_initial_checkpoint_delay(
        &mut self,
        room: RoomId,
    ) -> Result<TimeRange, SignalingModuleError>;

    async fn get_checkpoint_interval(
        &mut self,
        room: RoomId,
    ) -> Result<TimeRange, SignalingModuleError>;

    async fn get_next_checkpoint(
        &mut self,
        room: RoomId,
    ) -> Result<Option<Timestamp>, SignalingModuleError>;

    async fn add_known_participant(
        &mut self,
        room: RoomId,
        participant: ParticipantId,
    ) -> Result<(), SignalingModuleError>;

    async fn switch_to_next_checkpoint(
        &mut self,
        room: RoomId,
        new_next_checkpoint: Timestamp,
    ) -> Result<(), SignalingModuleError>;

    async fn record_presence_confirmation(
        &mut self,
        room: RoomId,
        participant: ParticipantId,
        timestamp: Timestamp,
    ) -> Result<(), SignalingModuleError>;

    async fn get_recorded_presence_state(
        &mut self,
        room: RoomId,
        participant: ParticipantId,
    ) -> Result<ParticipationLoggingState, SignalingModuleError>;
}
