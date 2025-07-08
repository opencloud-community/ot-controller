// SPDX-FileCopyrightText: OpenTalk GmbH <mail@opentalk.eu>
//
// SPDX-License-Identifier: EUPL-1.2

use std::collections::{BTreeMap, BTreeSet};

use async_trait::async_trait;
use opentalk_signaling_core::{NotFoundSnafu, RedisConnection, RedisSnafu, SignalingModuleError};
use opentalk_types_common::{
    rooms::RoomId,
    time::Timestamp,
    training_participation_report::{TimeRange, TrainingParticipationReportParameterSet},
};
use opentalk_types_signaling::ParticipantId;
use opentalk_types_signaling_training_participation_report::state::ParticipationLoggingState;
use redis::{AsyncCommands, ExistenceCheck, SetOptions};
use redis_args::{FromRedisValue, ToRedisArgs};
use serde::{Deserialize, Serialize};
use snafu::{OptionExt as _, ResultExt as _, ensure_whatever, whatever};

use super::{Checkpoint, RoomState, TrainingParticipationReportStorage, TrainingReportState};

#[async_trait(?Send)]
impl TrainingParticipationReportStorage for RedisConnection {
    async fn set_parameter_set_initialized(
        &mut self,
        room: RoomId,
    ) -> Result<(), SignalingModuleError> {
        self.set(RoomIsParameterSetInitializedKey { room }, true)
            .await
            .context(RedisSnafu {
                message: "Failed to SET parameter set initialized flag",
            })
    }

    async fn is_parameter_set_initialized(
        &mut self,
        room: RoomId,
    ) -> Result<bool, SignalingModuleError> {
        self.get(RoomIsParameterSetInitializedKey { room })
            .await
            .context(RedisSnafu {
                message: "Failed to GET parameter set initialized flag",
            })
    }

    async fn delete_parameter_set_initialized(
        &mut self,
        room: RoomId,
    ) -> Result<(), SignalingModuleError> {
        self.del(RoomIsParameterSetInitializedKey { room })
            .await
            .context(RedisSnafu {
                message: "Failed to DEL parameter set initialized flag",
            })
    }

    async fn get_parameter_set(
        &mut self,
        room: RoomId,
    ) -> Result<Option<TrainingParticipationReportParameterSet>, SignalingModuleError> {
        self.get(RoomParameterSetKey { room })
            .await
            .context(RedisSnafu {
                message: "Failed to GET parameter set",
            })
    }

    async fn set_parameter_set(
        &mut self,
        room: RoomId,
        value: TrainingParticipationReportParameterSet,
    ) -> Result<(), SignalingModuleError> {
        self.set(RoomParameterSetKey { room }, value)
            .await
            .context(RedisSnafu {
                message: "Failed to SET parameter set",
            })
    }

    async fn delete_parameter_set(&mut self, room: RoomId) -> Result<(), SignalingModuleError> {
        self.del(RoomParameterSetKey { room })
            .await
            .context(RedisSnafu {
                message: "Failed to DEL parameter set",
            })
    }

    async fn initialize_room(
        &mut self,
        room: RoomId,
        start: Timestamp,
        report_state: TrainingReportState,
        initial_checkpoint_delay: TimeRange,
        checkpoint_interval: TimeRange,
        known_participants: BTreeSet<ParticipantId>,
    ) -> Result<(), SignalingModuleError> {
        let mut pipe = redis::pipe();
        _ = pipe
            .atomic()
            .set_nx(
                StaticRoomInformationKey { room },
                StaticRoomInformation {
                    start,
                    initial_checkpoint_delay,
                    checkpoint_interval,
                },
            )
            .set_nx(TrainingReportStateKey { room }, report_state)
            .set_nx(
                NextCheckpointKey { room },
                NextCheckpoint {
                    next_checkpoint: None,
                },
            )
            .lpush(CheckpointEntriesKey { room }, CheckpointEntry::Start);
        for participant in known_participants {
            _ = pipe.sadd(KnownParticipantsKey { room }, participant);
        }

        pipe.exec_async(self).await.context(RedisSnafu {
            message: "failed to initialize training participation report room state",
        })?;
        Ok(())
    }

    async fn cleanup_room(
        &mut self,
        room: RoomId,
    ) -> Result<Option<RoomState>, SignalingModuleError> {
        match redis::pipe()
            .atomic()
            .get_del(StaticRoomInformationKey { room })
            .get_del(TrainingReportStateKey { room })
            .get_del(NextCheckpointKey { room })
            .smembers(KnownParticipantsKey { room })
            .del(KnownParticipantsKey { room })
            .lrange(CheckpointEntriesKey { room }, 0, -1)
            .del(CheckpointEntriesKey { room })
            .query_async::<(
                _,
                _,
                _,
                BTreeSet<ParticipantId>,
                (),
                Vec<CheckpointEntry>,
                (),
            )>(self)
            .await
            .context(RedisSnafu {
                message: "failed to cleanup training participation report room state",
            })? {
            (
                Some(StaticRoomInformation {
                    start,
                    initial_checkpoint_delay,
                    checkpoint_interval,
                }),
                Some(report_state),
                Some(NextCheckpoint { next_checkpoint }),
                known_participants,
                _,
                checkpoint_entries,
                _,
            ) => Ok(Some(RoomState {
                start,
                report_state,
                initial_checkpoint_delay,
                checkpoint_interval,
                history: collect_checkpoints(checkpoint_entries.into_iter().rev()),
                next_checkpoint,
                known_participants,
            })),
            (None, None, None, _, _, _, _) => Ok(None),
            _ => whatever!("inconsistent training participation report room state found on redis"),
        }
    }

    async fn get_training_report_state(
        &mut self,
        room: RoomId,
    ) -> Result<Option<TrainingReportState>, SignalingModuleError> {
        self.get(TrainingReportStateKey { room })
            .await
            .context(RedisSnafu {
                message: "failed to get training participation report room state",
            })
    }

    async fn set_training_report_state(
        &mut self,
        room: RoomId,
        report_state: TrainingReportState,
    ) -> Result<(), SignalingModuleError> {
        let response: Option<()> = self
            .set_options(
                TrainingReportStateKey { room },
                report_state,
                SetOptions::default().conditional_set(ExistenceCheck::XX),
            )
            .await
            .context(RedisSnafu {
                message: "failed to set training participation report room state",
            })?;
        response.context(NotFoundSnafu {
            message: "failed to update training report state value because it is not set",
        })
    }

    async fn get_initial_checkpoint_delay(
        &mut self,
        room: RoomId,
    ) -> Result<TimeRange, SignalingModuleError> {
        let information: StaticRoomInformation = self
            .get(StaticRoomInformationKey { room })
            .await
            .context(RedisSnafu {
                message: "failed to get training participation report initial checkpoint delay",
            })?;
        Ok(information.initial_checkpoint_delay)
    }

    async fn get_checkpoint_interval(
        &mut self,
        room: RoomId,
    ) -> Result<TimeRange, SignalingModuleError> {
        let information: StaticRoomInformation = self
            .get(StaticRoomInformationKey { room })
            .await
            .context(RedisSnafu {
                message: "failed to get training participation report checkpoint interval",
            })?;
        Ok(information.checkpoint_interval)
    }

    async fn get_next_checkpoint(
        &mut self,
        room: RoomId,
    ) -> Result<Option<Timestamp>, SignalingModuleError> {
        let checkpoint_index: NextCheckpoint =
            self.get(NextCheckpointKey { room })
                .await
                .context(RedisSnafu {
                    message: "failed to get training participation report initial checkpoint delay",
                })?;
        Ok(checkpoint_index.next_checkpoint)
    }

    async fn add_known_participant(
        &mut self,
        room: RoomId,
        participant: ParticipantId,
    ) -> Result<(), SignalingModuleError> {
        self.sadd(KnownParticipantsKey { room }, participant)
            .await
            .context(RedisSnafu {
                message: "failed to add entry to list of known participants",
            })
    }

    async fn switch_to_next_checkpoint(
        &mut self,
        room: RoomId,
        new_next_checkpoint: Timestamp,
    ) -> Result<(), SignalingModuleError> {
        let current_next_checkpoint = self.get_next_checkpoint(room).await?;

        let mut pipe = redis::pipe();
        _ = pipe.atomic().set_options(
            NextCheckpointKey { room },
            NextCheckpoint {
                next_checkpoint: Some(new_next_checkpoint),
            },
            SetOptions::default().conditional_set(ExistenceCheck::XX),
        );
        if let Some(timestamp) = current_next_checkpoint {
            _ = pipe.lpush_exists(
                CheckpointEntriesKey { room },
                CheckpointEntry::NextCheckpoint { timestamp },
            );
        }
        pipe.exec_async(self).await.context(RedisSnafu {
            message: "failed to switch to next checkpoint",
        })
    }

    async fn record_presence_confirmation(
        &mut self,
        room: RoomId,
        participant: ParticipantId,
        timestamp: Timestamp,
    ) -> Result<(), SignalingModuleError> {
        let training_report_state: Option<TrainingReportState> = self
            .get(TrainingReportStateKey { room })
            .await
            .context(RedisSnafu {
                message: "failed to retrieve room training report state",
            })?;
        ensure_whatever!(
            training_report_state == Some(TrainingReportState::TrackingPresence),
            "failed to record presence confirmation because room is not in tracking presence state"
        );
        let last_checkpoint: Vec<CheckpointEntry> = self
            .lrange(CheckpointEntriesKey { room }, 0, 0)
            .await
            .context(RedisSnafu {
                message: "failed to load checkpoint entries for room",
            })?;
        ensure_whatever!(
            matches!(
                last_checkpoint.first(),
                Some(CheckpointEntry::NextCheckpoint { .. })
                    | Some(CheckpointEntry::Participation { .. })
            ),
            "cannot record presence confirmation because no checkpoint has been found"
        );

        let () = self
            .lpush_exists(
                CheckpointEntriesKey { room },
                CheckpointEntry::Participation {
                    participant,
                    timestamp,
                },
            )
            .await
            .context(RedisSnafu {
                message: "failed to record presence confirmation",
            })?;
        Ok(())
    }

    async fn get_recorded_presence_state(
        &mut self,
        room: RoomId,
        participant: ParticipantId,
    ) -> Result<ParticipationLoggingState, SignalingModuleError> {
        let (entries, state): (Vec<CheckpointEntry>, Option<TrainingReportState>) = redis::pipe()
            .lrange(CheckpointEntriesKey { room }, 0, -1)
            .get(TrainingReportStateKey { room })
            .query_async(self)
            .await
            .context(RedisSnafu {
                message: "failed to load checkpoint entries for room",
            })?;
        let tracking_presence = state == Some(TrainingReportState::TrackingPresence);
        let enabled = !entries.is_empty();
        let history = collect_checkpoints(entries.into_iter().rev());
        match history.last() {
            Some(Checkpoint { presence, .. }) if presence.contains_key(&participant) => {
                Ok(ParticipationLoggingState::Enabled)
            }
            Some(Checkpoint { .. }) => {
                if tracking_presence {
                    Ok(ParticipationLoggingState::WaitingForConfirmation)
                } else {
                    Ok(ParticipationLoggingState::Enabled)
                }
            }
            None if enabled => Ok(ParticipationLoggingState::Enabled),
            None => Ok(ParticipationLoggingState::Disabled),
        }
    }
}

#[derive(ToRedisArgs)]
#[to_redis_args(
    fmt = "opentalk-signaling:room={room}:training_report:is_parameter_set_initialized"
)]
struct RoomIsParameterSetInitializedKey {
    room: RoomId,
}

#[derive(ToRedisArgs)]
#[to_redis_args(fmt = "opentalk-signaling:room={room}:training_report:parameter_set")]
struct RoomParameterSetKey {
    room: RoomId,
}

#[derive(Debug, Clone, PartialEq, Eq, Serialize, Deserialize, ToRedisArgs, FromRedisValue)]
#[to_redis_args(serde)]
#[from_redis_value(serde)]
struct StaticRoomInformation {
    start: Timestamp,
    initial_checkpoint_delay: TimeRange,
    checkpoint_interval: TimeRange,
}

#[derive(Debug, Clone, PartialEq, Eq, Serialize, Deserialize, ToRedisArgs, FromRedisValue)]
#[to_redis_args(serde)]
#[from_redis_value(serde)]
struct NextCheckpoint {
    next_checkpoint: Option<Timestamp>,
}

#[derive(ToRedisArgs)]
#[to_redis_args(fmt = "opentalk-signaling:room={room}:training_report:static_room_information")]
struct StaticRoomInformationKey {
    room: RoomId,
}

#[derive(ToRedisArgs)]
#[to_redis_args(fmt = "opentalk-signaling:room={room}:training_report:state")]
struct TrainingReportStateKey {
    room: RoomId,
}

#[derive(ToRedisArgs)]
#[to_redis_args(fmt = "opentalk-signaling:room={room}:training_report:next_checkpoint")]
struct NextCheckpointKey {
    room: RoomId,
}

#[derive(ToRedisArgs)]
#[to_redis_args(fmt = "opentalk-signaling:room={room}:training_report:checkpoint_entries")]
struct CheckpointEntriesKey {
    room: RoomId,
}

#[derive(ToRedisArgs)]
#[to_redis_args(fmt = "opentalk-signaling:room={room}:training_report:known_participants")]
struct KnownParticipantsKey {
    room: RoomId,
}

#[derive(Debug, Clone, PartialEq, Eq, Serialize, Deserialize, ToRedisArgs, FromRedisValue)]
#[to_redis_args(serde)]
#[from_redis_value(serde)]
#[serde(tag = "entry")]
enum CheckpointEntry {
    Start,
    NextCheckpoint {
        timestamp: Timestamp,
    },
    Participation {
        participant: ParticipantId,
        timestamp: Timestamp,
    },
}

fn collect_checkpoints(entries: impl Iterator<Item = CheckpointEntry>) -> Vec<Checkpoint> {
    let mut checkpoints = Vec::new();

    for entry in entries {
        match entry {
            CheckpointEntry::Start => {}
            CheckpointEntry::NextCheckpoint { timestamp } => {
                checkpoints.push(Checkpoint {
                    timestamp,
                    presence: BTreeMap::new(),
                });
            }
            CheckpointEntry::Participation {
                participant,
                timestamp,
            } => {
                if let Some(checkpoint) = checkpoints.last_mut() {
                    _ = checkpoint.presence.insert(participant, timestamp);
                }
            }
        }
    }

    checkpoints
}

#[cfg(test)]
mod tests {
    use opentalk_signaling_core::RedisConnection;
    use redis::aio::ConnectionManager;
    use serial_test::serial;

    use crate::storage::test_common;

    async fn storage() -> RedisConnection {
        let redis_url =
            std::env::var("REDIS_ADDR").unwrap_or_else(|_| "redis://0.0.0.0:6379/".to_owned());
        let redis = redis::Client::open(redis_url).expect("Invalid redis url");

        let mut mgr = ConnectionManager::new(redis).await.unwrap();

        redis::cmd("FLUSHALL").exec_async(&mut mgr).await.unwrap();

        RedisConnection::new(mgr)
    }

    #[tokio::test]
    #[serial]
    async fn parameter_set_initialized() {
        test_common::parameter_set_initialized(&mut storage().await).await;
    }

    #[tokio::test]
    #[serial]
    async fn parameter_set() {
        test_common::parameter_set(&mut storage().await).await;
    }

    #[tokio::test]
    #[serial]
    async fn initialize_room_and_cleanup() {
        test_common::initialize_room_and_cleanup(&mut storage().await).await;
    }

    #[tokio::test]
    #[serial]
    async fn get_set_training_report_state() {
        test_common::get_set_training_report_state(&mut storage().await).await;
    }

    #[tokio::test]
    #[serial]
    async fn get_initial_checkpoint_delay() {
        test_common::get_initial_checkpoint_delay(&mut storage().await).await;
    }

    #[tokio::test]
    #[serial]
    async fn get_checkpoint_interval() {
        test_common::get_checkpoint_interval(&mut storage().await).await;
    }

    #[tokio::test]
    #[serial]
    async fn get_and_switch_to_next_checkpoint() {
        test_common::get_and_switch_to_next_checkpoint(&mut storage().await).await;
    }

    #[tokio::test]
    #[serial]
    async fn record_presence() {
        test_common::record_presence(&mut storage().await).await;
    }
}
