// SPDX-FileCopyrightText: OpenTalk GmbH <mail@opentalk.eu>
//
// SPDX-License-Identifier: EUPL-1.2

use std::{
    collections::BTreeSet,
    sync::{Arc, OnceLock},
};

use async_trait::async_trait;
use opentalk_signaling_core::{SignalingModuleError, VolatileStaticMemoryStorage};
use opentalk_types_common::{rooms::RoomId, time::Timestamp};
use opentalk_types_signaling::ParticipantId;
use opentalk_types_signaling_training_participation_report::{
    state::ParticipationLoggingState, TimeRange,
};
use parking_lot::RwLock;

use super::memory::TrainingParticipationReportState;
use crate::storage::{RoomState, TrainingParticipationReportStorage, TrainingReportState};

static STATE: OnceLock<Arc<RwLock<TrainingParticipationReportState>>> = OnceLock::new();

fn state() -> &'static Arc<RwLock<TrainingParticipationReportState>> {
    STATE.get_or_init(Default::default)
}

#[async_trait(?Send)]
impl TrainingParticipationReportStorage for VolatileStaticMemoryStorage {
    async fn initialize_room(
        &mut self,
        room: RoomId,
        start: Timestamp,
        report_state: TrainingReportState,
        initial_checkpoint_delay: TimeRange,
        checkpoint_interval: TimeRange,
        known_participants: BTreeSet<ParticipantId>,
    ) -> Result<(), SignalingModuleError> {
        state().write().initialize_room(
            room,
            start,
            report_state,
            initial_checkpoint_delay,
            checkpoint_interval,
            known_participants,
        );
        Ok(())
    }

    async fn cleanup_room(
        &mut self,
        room: RoomId,
    ) -> Result<Option<RoomState>, SignalingModuleError> {
        Ok(state().write().cleanup_room(room))
    }

    async fn get_training_report_state(
        &mut self,
        room: RoomId,
    ) -> Result<Option<TrainingReportState>, SignalingModuleError> {
        state().read().get_training_report_state(room)
    }

    async fn set_training_report_state(
        &mut self,
        room: RoomId,
        report_state: TrainingReportState,
    ) -> Result<(), SignalingModuleError> {
        state()
            .write()
            .set_training_report_state(room, report_state)
    }

    async fn get_initial_checkpoint_delay(
        &mut self,
        room: RoomId,
    ) -> Result<TimeRange, SignalingModuleError> {
        state().read().get_initial_checkpoint_delay(room)
    }

    async fn get_checkpoint_interval(
        &mut self,
        room: RoomId,
    ) -> Result<TimeRange, SignalingModuleError> {
        state().read().get_checkpoint_interval(room)
    }

    async fn get_next_checkpoint(
        &mut self,
        room: RoomId,
    ) -> Result<Option<Timestamp>, SignalingModuleError> {
        state().read().get_next_checkpoint(room)
    }

    async fn add_known_participant(
        &mut self,
        room: RoomId,
        participant: ParticipantId,
    ) -> Result<(), SignalingModuleError> {
        state().write().add_known_participant(room, participant)
    }

    async fn switch_to_next_checkpoint(
        &mut self,
        room: RoomId,
        new_next_checkpoint: Timestamp,
    ) -> Result<(), SignalingModuleError> {
        state()
            .write()
            .switch_to_next_checkpoint(room, new_next_checkpoint)
    }

    async fn record_presence_confirmation(
        &mut self,
        room: RoomId,
        participant: ParticipantId,
        timestamp: Timestamp,
    ) -> Result<(), SignalingModuleError> {
        state()
            .write()
            .record_presence_confirmation(room, participant, timestamp)
    }

    async fn get_recorded_presence_state(
        &mut self,
        room: RoomId,
        participant: ParticipantId,
    ) -> Result<ParticipationLoggingState, SignalingModuleError> {
        state()
            .read()
            .get_recorded_presence_state(room, participant)
    }
}

#[cfg(test)]
mod tests {
    use opentalk_signaling_core::VolatileStaticMemoryStorage;
    use serial_test::serial;

    use super::{super::super::test_common, state};

    fn storage() -> VolatileStaticMemoryStorage {
        state().write().reset();
        VolatileStaticMemoryStorage
    }

    #[tokio::test]
    #[serial]
    async fn initialize_room_and_cleanup() {
        test_common::initialize_room_and_cleanup(&mut storage()).await;
    }

    #[tokio::test]
    #[serial]
    async fn get_set_training_report_state() {
        test_common::get_set_training_report_state(&mut storage()).await;
    }

    #[tokio::test]
    #[serial]
    async fn get_initial_checkpoint_delay() {
        test_common::get_initial_checkpoint_delay(&mut storage()).await;
    }

    #[tokio::test]
    #[serial]
    async fn get_checkpoint_interval() {
        test_common::get_checkpoint_interval(&mut storage()).await;
    }

    #[tokio::test]
    #[serial]
    async fn get_and_switch_to_next_checkpoint() {
        test_common::get_and_switch_to_next_checkpoint(&mut storage()).await;
    }

    #[tokio::test]
    #[serial]
    async fn record_presence() {
        test_common::record_presence(&mut storage()).await;
    }
}
