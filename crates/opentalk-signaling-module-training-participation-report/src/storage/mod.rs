// SPDX-FileCopyrightText: OpenTalk GmbH <mail@opentalk.eu>
//
// SPDX-License-Identifier: EUPL-1.2

mod checkpoint;
mod redis;
mod room_state;
mod training_participation_report_storage;
mod training_report_state;
mod volatile;

pub(crate) use checkpoint::Checkpoint;
pub(crate) use room_state::RoomState;
pub(crate) use training_participation_report_storage::TrainingParticipationReportStorage;
pub(crate) use training_report_state::TrainingReportState;

#[cfg(test)]
mod test_common {
    use std::collections::{BTreeMap, BTreeSet};

    use opentalk_signaling_core::SignalingModuleError;
    use opentalk_types_common::{
        rooms::RoomId,
        training_participation_report::{TimeRange, TrainingParticipationReportParameterSet},
    };
    use opentalk_types_signaling::ParticipantId;
    use opentalk_types_signaling_training_participation_report::state::ParticipationLoggingState;
    use pretty_assertions::assert_eq;

    use super::TrainingParticipationReportStorage;
    use crate::storage::{Checkpoint, RoomState, TrainingReportState};

    const ALICE: ParticipantId = ParticipantId::from_u128(0xd3cfaa81_23b5_4617_ba72_07db063cc72e);
    const BOB: ParticipantId = ParticipantId::from_u128(0x02ce458e_4fae_459d_87d6_045d62eb4f40);
    const CHARLIE: ParticipantId = ParticipantId::from_u128(0x26d15b4c_cb55_4ccf_b8df_7c821e98517b);

    pub(super) async fn parameter_set_initialized(
        storage: &mut dyn TrainingParticipationReportStorage,
    ) {
        let room = RoomId::generate();

        assert!(!storage.is_parameter_set_initialized(room).await.unwrap());

        storage.set_parameter_set_initialized(room).await.unwrap();

        assert!(storage.is_parameter_set_initialized(room).await.unwrap());

        storage
            .delete_parameter_set_initialized(room)
            .await
            .unwrap();

        assert!(!storage.is_parameter_set_initialized(room).await.unwrap());
    }

    pub(super) async fn parameter_set(storage: &mut dyn TrainingParticipationReportStorage) {
        let room = RoomId::generate();

        let parameter_set = TrainingParticipationReportParameterSet {
            initial_checkpoint_delay: TimeRange {
                after: 100,
                within: 200,
            },
            checkpoint_interval: TimeRange {
                after: 300,
                within: 400,
            },
        };

        assert!(storage.get_parameter_set(room).await.unwrap().is_none());

        storage
            .set_parameter_set(room, parameter_set.clone())
            .await
            .unwrap();

        assert_eq!(
            Some(parameter_set),
            storage.get_parameter_set(room).await.unwrap()
        );

        storage.delete_parameter_set(room).await.unwrap();

        assert!(storage.get_parameter_set(room).await.unwrap().is_none());
    }

    pub(super) async fn initialize_room_and_cleanup(
        storage: &mut dyn TrainingParticipationReportStorage,
    ) {
        let room = RoomId::generate();
        let start = "2025-01-01T01:01:01Z"
            .parse()
            .expect("value must be parsable as Timestamp");
        let report_state = TrainingReportState::WaitingForInitialTimeout;
        let initial_checkpoint_delay = TimeRange {
            after: 10.try_into().expect("value must be a positive number"),
            within: 20,
        };
        let checkpoint_interval = TimeRange {
            after: 30.try_into().expect("value must be a positive number"),
            within: 40,
        };

        let known_participants = BTreeSet::from_iter([ALICE, BOB]);

        storage
            .initialize_room(
                room,
                start,
                report_state,
                initial_checkpoint_delay.clone(),
                checkpoint_interval.clone(),
                known_participants.clone(),
            )
            .await
            .unwrap();

        assert_eq!(
            storage.cleanup_room(room).await.unwrap(),
            Some(RoomState {
                start,
                report_state,
                initial_checkpoint_delay,
                checkpoint_interval,
                history: vec![],
                next_checkpoint: None,
                known_participants,
            })
        );
        assert_eq!(storage.cleanup_room(room).await.unwrap(), None);
    }

    async fn initialize_room_example(
        storage: &mut dyn TrainingParticipationReportStorage,
        room: RoomId,
    ) -> Result<(), SignalingModuleError> {
        let start = "2025-01-01T01:01:01Z"
            .parse()
            .expect("value must be parsable as Timestamp");
        let report_state = TrainingReportState::WaitingForInitialTimeout;
        let initial_checkpoint_delay = TimeRange {
            after: 10.try_into().expect("value must be a positive number"),
            within: 20,
        };
        let checkpoint_interval = TimeRange {
            after: 30.try_into().expect("value must be a positive number"),
            within: 40,
        };
        let known_participants = BTreeSet::from_iter([ALICE, BOB]);

        storage
            .initialize_room(
                room,
                start,
                report_state,
                initial_checkpoint_delay.clone(),
                checkpoint_interval.clone(),
                known_participants.clone(),
            )
            .await?;
        Ok(())
    }

    pub(super) async fn get_set_training_report_state(
        storage: &mut dyn TrainingParticipationReportStorage,
    ) {
        let room = RoomId::generate();
        assert_eq!(storage.get_training_report_state(room).await.unwrap(), None);
        assert!(storage
            .set_training_report_state(room, TrainingReportState::WaitingForInitialTimeout)
            .await
            .is_err());

        initialize_room_example(storage, room).await.unwrap();

        assert!(storage.get_training_report_state(room).await.is_ok());

        storage
            .set_training_report_state(room, TrainingReportState::WaitingForInitialTimeout)
            .await
            .unwrap();
        assert_eq!(
            storage.get_training_report_state(room).await.unwrap(),
            Some(TrainingReportState::WaitingForInitialTimeout)
        );

        storage
            .set_training_report_state(room, TrainingReportState::TrackingPresence)
            .await
            .unwrap();
        assert_eq!(
            storage.get_training_report_state(room).await.unwrap(),
            Some(TrainingReportState::TrackingPresence)
        );

        _ = storage.cleanup_room(room).await.unwrap();
        assert_eq!(storage.get_training_report_state(room).await.unwrap(), None);
    }

    pub(super) async fn get_initial_checkpoint_delay(
        storage: &mut dyn TrainingParticipationReportStorage,
    ) {
        let room = RoomId::generate();
        assert!(storage.get_initial_checkpoint_delay(room).await.is_err());

        initialize_room_example(storage, room).await.unwrap();

        assert_eq!(
            storage.get_initial_checkpoint_delay(room).await.unwrap(),
            TimeRange {
                after: 10.try_into().unwrap(),
                within: 20
            }
        );

        _ = storage.cleanup_room(room).await.unwrap();
        assert!(storage.get_initial_checkpoint_delay(room).await.is_err());
    }

    pub(super) async fn get_checkpoint_interval(
        storage: &mut dyn TrainingParticipationReportStorage,
    ) {
        let room = RoomId::generate();
        assert!(storage.get_checkpoint_interval(room).await.is_err());

        initialize_room_example(storage, room).await.unwrap();

        assert_eq!(
            storage.get_checkpoint_interval(room).await.unwrap(),
            TimeRange {
                after: 30.try_into().unwrap(),
                within: 40
            }
        );

        _ = storage.cleanup_room(room).await.unwrap();
        assert!(storage.get_checkpoint_interval(room).await.is_err());
    }

    pub(super) async fn get_and_switch_to_next_checkpoint(
        storage: &mut dyn TrainingParticipationReportStorage,
    ) {
        let room = RoomId::generate();
        assert!(storage.get_next_checkpoint(room).await.is_err());

        let checkpoint1 = "2025-02-03T01:01:01Z"
            .parse()
            .expect("value must be parsable as Timestamp");
        let checkpoint2 = "2025-04-05T01:01:01Z"
            .parse()
            .expect("value must be parsable as Timestamp");

        initialize_room_example(storage, room).await.unwrap();

        assert_eq!(storage.get_next_checkpoint(room).await.unwrap(), None);

        storage
            .switch_to_next_checkpoint(room, checkpoint1)
            .await
            .unwrap();
        assert_eq!(
            storage.get_next_checkpoint(room).await.unwrap(),
            Some(checkpoint1)
        );

        storage
            .switch_to_next_checkpoint(room, checkpoint2)
            .await
            .unwrap();
        assert_eq!(
            storage.get_next_checkpoint(room).await.unwrap(),
            Some(checkpoint2)
        );

        _ = storage.cleanup_room(room).await.unwrap();
        assert!(storage.get_next_checkpoint(room).await.is_err());
    }

    pub(super) async fn record_presence(storage: &mut dyn TrainingParticipationReportStorage) {
        let room = RoomId::generate();

        let checkpoint1 = "2025-02-03T01:01:01Z"
            .parse()
            .expect("value must be parsable as Timestamp");
        let recorded1a = "2025-02-03T01:01:02Z"
            .parse()
            .expect("value must be parsable as Timestamp");
        let recorded1b = "2025-02-03T01:01:03Z"
            .parse()
            .expect("value must be parsable as Timestamp");

        let checkpoint2 = "2025-04-05T01:01:01Z"
            .parse()
            .expect("value must be parsable as Timestamp");
        let recorded2b = "2025-04-05T01:01:03Z"
            .parse()
            .expect("value must be parsable as Timestamp");

        let checkpoint3 = "2025-06-07T01:01:01Z"
            .parse()
            .expect("value must be parsable as Timestamp");

        assert_eq!(
            storage
                .get_recorded_presence_state(room, ALICE)
                .await
                .unwrap(),
            ParticipationLoggingState::Disabled
        );

        initialize_room_example(storage, room).await.unwrap();

        // Setting the stage
        {
            // Returns an error because we're not yet in `TrackingPresence` state
            assert!(storage
                .record_presence_confirmation(room, ALICE, recorded1a)
                .await
                .is_err());
            assert_eq!(
                storage
                    .get_recorded_presence_state(room, ALICE)
                    .await
                    .unwrap(),
                ParticipationLoggingState::Enabled
            );

            storage
                .set_training_report_state(room, TrainingReportState::TrackingPresence)
                .await
                .unwrap();

            // Returns an error because even though the state is correct we
            // don't have a current checkpoint yet
            assert!(storage
                .record_presence_confirmation(room, ALICE, recorded1a)
                .await
                .is_err());

            storage
                .switch_to_next_checkpoint(room, checkpoint1)
                .await
                .unwrap();

            // Returns an error because even though the state is correct we
            // don't have a current checkpoint yet, we only queued the timestamp
            // for the upcoming checkpoint.
            assert!(storage
                .record_presence_confirmation(room, ALICE, recorded1a)
                .await
                .is_err());
        }

        // Record the first checkpoint
        {
            assert_eq!(
                storage
                    .get_recorded_presence_state(room, ALICE)
                    .await
                    .unwrap(),
                ParticipationLoggingState::Enabled
            );

            // We want to record presence for checkpoint 1 now, but the next
            // checkpoint timestamp after that must be queued for checkpoint 1
            // to become the current checkpoint.
            storage
                .switch_to_next_checkpoint(room, checkpoint2)
                .await
                .unwrap();

            assert_eq!(
                storage
                    .get_recorded_presence_state(room, ALICE)
                    .await
                    .unwrap(),
                ParticipationLoggingState::WaitingForConfirmation
            );
            storage
                .record_presence_confirmation(room, ALICE, recorded1a)
                .await
                .unwrap();
            assert_eq!(
                storage
                    .get_recorded_presence_state(room, ALICE)
                    .await
                    .unwrap(),
                ParticipationLoggingState::Enabled
            );

            storage
                .record_presence_confirmation(room, BOB, recorded1b)
                .await
                .unwrap();
        }

        // Record the second checkpoint
        {
            assert_eq!(
                storage
                    .get_recorded_presence_state(room, ALICE)
                    .await
                    .unwrap(),
                ParticipationLoggingState::Enabled
            );

            // We want to record presence for checkpoint 2 now, but the next
            // checkpoint timestamp after that must be queued for checkpoint 2
            // to become the current checkpoint.
            storage
                .switch_to_next_checkpoint(room, checkpoint3)
                .await
                .unwrap();

            assert_eq!(
                storage
                    .get_recorded_presence_state(room, ALICE)
                    .await
                    .unwrap(),
                ParticipationLoggingState::WaitingForConfirmation
            );
            storage
                .record_presence_confirmation(room, BOB, recorded2b)
                .await
                .unwrap();
        }

        // Add another participant to the known participants list
        {
            storage.add_known_participant(room, CHARLIE).await.unwrap();
        }

        // Verify the recorded checkpoints and known participants
        {
            let room_state = storage
                .cleanup_room(room)
                .await
                .unwrap()
                .expect("room state must be present");

            assert_eq!(
                room_state.history,
                vec![
                    Checkpoint {
                        timestamp: checkpoint1,
                        presence: BTreeMap::from_iter([(ALICE, recorded1a), (BOB, recorded1b),])
                    },
                    Checkpoint {
                        timestamp: checkpoint2,
                        presence: BTreeMap::from_iter([(BOB, recorded2b),])
                    },
                ]
            );

            assert_eq!(
                room_state.known_participants,
                BTreeSet::from_iter([ALICE, BOB, CHARLIE])
            );
        }

        assert_eq!(
            storage
                .get_recorded_presence_state(room, ALICE)
                .await
                .unwrap(),
            ParticipationLoggingState::Disabled
        );
    }
}
