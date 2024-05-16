// SPDX-FileCopyrightText: OpenTalk GmbH <mail@opentalk.eu>
//
// SPDX-License-Identifier: EUPL-1.2

use std::{
    collections::{BTreeMap, BTreeSet, HashMap, HashSet},
    convert::identity,
    fmt::Debug,
    time::Duration,
};

use async_trait::async_trait;
use opentalk_db_storage::{events::Event, tariffs::Tariff};
use opentalk_r3dlock::Mutex;
use opentalk_types::{
    core::{ParticipantId, RoomId, Timestamp},
    signaling::Role,
};
use redis::{AsyncCommands, FromRedisValue, ToRedisArgs};
use redis_args::ToRedisArgs;
use snafu::ResultExt;

use super::{control_storage::ControlStorage, SKIP_WAITING_ROOM_KEY_EXPIRY};
use crate::{RedisConnection, RedisSnafu, SignalingModuleError, SignalingRoomId};

#[async_trait(?Send)]
impl ControlStorage for RedisConnection {
    #[tracing::instrument(level = "debug", skip(self))]
    async fn participant_set_exists(
        &mut self,
        room: SignalingRoomId,
    ) -> Result<bool, SignalingModuleError> {
        self.exists(RoomParticipants { room })
            .await
            .context(RedisSnafu {
                message: "Failed to check if participants exist",
            })
    }

    #[tracing::instrument(level = "debug", skip(self))]
    async fn get_all_participants(
        &mut self,
        room: SignalingRoomId,
    ) -> Result<BTreeSet<ParticipantId>, SignalingModuleError> {
        self.smembers(RoomParticipants { room })
            .await
            .context(RedisSnafu {
                message: "Failed to get participants",
            })
    }

    #[tracing::instrument(level = "debug", skip(self))]
    async fn remove_participant_set(
        &mut self,
        room: SignalingRoomId,
    ) -> Result<(), SignalingModuleError> {
        self.del(RoomParticipants { room })
            .await
            .context(RedisSnafu {
                message: "Failed to del participants",
            })
    }

    #[tracing::instrument(level = "debug", skip(self))]
    async fn participants_contains(
        &mut self,
        room: SignalingRoomId,
        participant: ParticipantId,
    ) -> Result<bool, SignalingModuleError> {
        self.sismember(RoomParticipants { room }, participant)
            .await
            .context(RedisSnafu {
                message: "Failed to check if participants contains participant",
            })
    }

    #[tracing::instrument(level = "debug", skip(self))]
    async fn check_participants_exist(
        &mut self,
        room: SignalingRoomId,
        participants: &[ParticipantId],
    ) -> Result<bool, SignalingModuleError> {
        let bools: Vec<bool> = redis::cmd("SMISMEMBER")
            .arg(RoomParticipants { room })
            .arg(participants)
            .query_async(self)
            .await
            .context(RedisSnafu {
                message: "Failed to check if participants contains participant",
            })?;

        Ok(bools.into_iter().all(identity))
    }

    #[tracing::instrument(level = "debug", skip(self))]
    async fn add_participant_to_set(
        &mut self,
        room: SignalingRoomId,
        participant: ParticipantId,
    ) -> Result<bool, SignalingModuleError> {
        self.sadd(RoomParticipants { room }, participant)
            .await
            .map(|num_added: usize| num_added > 0)
            .context(RedisSnafu {
                message: "Failed to add own participant id to set",
            })
    }

    #[tracing::instrument(level = "debug", skip(self))]
    async fn get_attribute<V>(
        &mut self,
        room: SignalingRoomId,
        participant: ParticipantId,
        name: &str,
    ) -> Result<V, SignalingModuleError>
    where
        V: FromRedisValue,
    {
        let value = self
            .hget(
                RoomParticipantAttributes {
                    room,
                    attribute_name: name,
                },
                participant,
            )
            .await
            .with_context(|_| RedisSnafu {
                message: format!("Failed to get attribute {name}"),
            })?;

        Ok(value)
    }

    #[tracing::instrument(level = "debug", skip(self))]
    async fn set_attribute<V>(
        &mut self,
        room: SignalingRoomId,
        participant: ParticipantId,
        name: &str,
        value: V,
    ) -> Result<(), SignalingModuleError>
    where
        V: Debug + ToRedisArgs + Send + Sync,
    {
        self.hset(
            RoomParticipantAttributes {
                room,
                attribute_name: name,
            },
            participant,
            value,
        )
        .await
        .with_context(|_| RedisSnafu {
            message: format!("Failed to set attribute {name}"),
        })?;

        Ok(())
    }

    #[tracing::instrument(level = "debug", skip(self))]
    async fn remove_attribute(
        &mut self,
        room: SignalingRoomId,
        participant: ParticipantId,
        name: &str,
    ) -> Result<(), SignalingModuleError> {
        self.hdel(
            RoomParticipantAttributes {
                room,
                attribute_name: name,
            },
            participant,
        )
        .await
        .with_context(|_| RedisSnafu {
            message: format!("Failed to remove participant attribute key, {name}"),
        })
    }

    #[tracing::instrument(level = "debug", skip(self))]
    async fn get_attribute_for_participants<V>(
        &mut self,
        room: SignalingRoomId,
        name: &str,
        participants: &[ParticipantId],
    ) -> Result<Vec<Option<V>>, SignalingModuleError>
    where
        V: FromRedisValue,
    {
        // Special case: HMGET cannot handle empty arrays (missing arguments)
        if participants.is_empty() {
            Ok(vec![])
        } else {
            // need manual HMGET command as the HGET command wont work with single value vector input
            redis::cmd("HMGET")
                .arg(RoomParticipantAttributes {
                    room,
                    attribute_name: name,
                })
                .arg(participants)
                .query_async(self)
                .await
                .with_context(|_| RedisSnafu {
                    message: format!("Failed to get attribute '{name}' for all participants "),
                })
        }
    }

    #[tracing::instrument(level = "debug", skip(self))]
    async fn remove_attribute_key(
        &mut self,
        room: SignalingRoomId,
        name: &str,
    ) -> Result<(), SignalingModuleError> {
        self.del(RoomParticipantAttributes {
            room,
            attribute_name: name,
        })
        .await
        .with_context(|_| RedisSnafu {
            message: format!("Failed to remove participant attribute key, {name}"),
        })
    }

    #[tracing::instrument(level = "debug", skip(self))]
    async fn get_role_and_left_at_for_room_participants(
        &mut self,
        room: SignalingRoomId,
    ) -> Result<BTreeMap<ParticipantId, (Option<Role>, Option<Timestamp>)>, SignalingModuleError>
    {
        let mut pipe = redis::pipe();
        pipe.atomic();
        pipe.hgetall(RoomParticipantAttributes {
            room,
            attribute_name: "role",
        });
        pipe.hgetall(RoomParticipantAttributes {
            room,
            attribute_name: "left_at",
        });

        let (mut roles, mut left_at_timestamps): (
            HashMap<ParticipantId, Role>,
            HashMap<ParticipantId, Timestamp>,
        ) = pipe.query_async(self).await.context(RedisSnafu {
            message: "Failed to get attributes",
        })?;
        let participants: HashSet<ParticipantId> = roles
            .keys()
            .chain(left_at_timestamps.keys())
            .copied()
            .collect();

        Ok(participants
            .into_iter()
            .map(|p| (p, (roles.remove(&p), left_at_timestamps.remove(&p))))
            .collect())
    }

    #[tracing::instrument(level = "debug", skip(self))]
    async fn try_init_tariff(
        &mut self,
        room_id: RoomId,
        tariff: Tariff,
    ) -> Result<Tariff, SignalingModuleError> {
        let (_, tariff): (bool, Tariff) = redis::pipe()
            .atomic()
            .set_nx(RoomTariff { room_id }, tariff)
            .get(RoomTariff { room_id })
            .query_async(self)
            .await
            .context(RedisSnafu {
                message: "Failed to SET NX & GET room tariff",
            })?;

        Ok(tariff)
    }
}

/// Describes a set of participants inside a room.
/// This MUST always be locked before accessing it
#[derive(ToRedisArgs)]
#[to_redis_args(fmt = "opentalk-signaling:room={room}:participants")]
struct RoomParticipants {
    room: SignalingRoomId,
}

/// Key used for the lock over the room participants set
#[derive(ToRedisArgs)]
#[to_redis_args(fmt = "opentalk-signaling:room={room}:participants.lock")]
pub struct RoomLock {
    pub room: SignalingRoomId,
}

/// Key used for the lock over the room participants set
#[derive(ToRedisArgs)]
#[to_redis_args(fmt = "opentalk-signaling:room={room}:participants:attributes:{attribute_name}")]
struct RoomParticipantAttributes<'s> {
    room: SignalingRoomId,
    attribute_name: &'s str,
}

/// The total count of all participants in the room, also considers participants in breakout rooms and the waiting room
///
/// Notice that this key only contains the [`RoomId`] as it applies to all breakout rooms as well
#[derive(ToRedisArgs)]
#[to_redis_args(fmt = "opentalk-signaling:room={room_id}:participant-count")]
pub struct RoomParticipantCount {
    room_id: RoomId,
}

/// The configured [`Tariff`] for the room
///
/// Notice that this key only contains the [`RoomId`] as it applies to all breakout rooms as well
#[derive(ToRedisArgs)]
#[to_redis_args(fmt = "opentalk-signaling:room={room_id}:tariff")]
pub struct RoomTariff {
    room_id: RoomId,
}

/// The associated [`Event`] for the room
///
/// Notice that this key only contains the [`RoomId`] as it applies to all breakout rooms as well
#[derive(ToRedisArgs)]
#[to_redis_args(fmt = "opentalk-signaling:room={room_id}:event")]
pub struct RoomEvent {
    room_id: RoomId,
}

/// The point in time the room closes.
#[derive(ToRedisArgs)]
#[to_redis_args(fmt = "opentalk-signaling:room={room}:closes_at")]
struct RoomClosesAt {
    room: SignalingRoomId,
}

/// The room's mutex
///
/// Must be taken when joining and leaving the room.
/// This allows for cleanups when the last user leaves without anyone joining.
///
/// The redlock parameters are set a bit higher than usual to combat contention when a room gets
/// destroyed while a large number of participants are inside it. (e.g. when a breakout room ends)
pub fn room_mutex(room: SignalingRoomId) -> Mutex<RoomLock> {
    Mutex::new(RoomLock { room })
        .with_wait_time(Duration::from_millis(20)..Duration::from_millis(60))
        .with_retries(20)
}

pub struct AttrPipeline {
    room: SignalingRoomId,
    participant: ParticipantId,
    pipe: redis::Pipeline,
}

// FIXME: Make the type inference better. e.g. by passing the type to get and letting get extend the final type.
impl AttrPipeline {
    pub fn new(room: SignalingRoomId, participant: ParticipantId) -> Self {
        let mut pipe = redis::pipe();
        pipe.atomic();

        Self {
            room,
            participant,
            pipe: redis::pipe(),
        }
    }

    pub fn set<V: ToRedisArgs>(&mut self, name: &str, value: V) -> &mut Self {
        self.pipe
            .hset(
                RoomParticipantAttributes {
                    room: self.room,
                    attribute_name: name,
                },
                self.participant,
                value,
            )
            .ignore();

        self
    }

    pub fn get(&mut self, name: &str) -> &mut Self {
        self.pipe.hget(
            RoomParticipantAttributes {
                room: self.room,
                attribute_name: name,
            },
            self.participant,
        );

        self
    }

    pub fn del(&mut self, name: &str) -> &mut Self {
        self.pipe
            .hdel(
                RoomParticipantAttributes {
                    room: self.room,
                    attribute_name: name,
                },
                self.participant,
            )
            .ignore();

        self
    }

    pub async fn query_async<T: FromRedisValue>(
        &mut self,
        redis_conn: &mut RedisConnection,
    ) -> redis::RedisResult<T> {
        self.pipe.query_async(redis_conn).await
    }
}

#[derive(Debug, ToRedisArgs)]
#[to_redis_args(fmt = "opentalk-signaling:runner:{id}")]
pub struct ParticipantIdRunnerLock {
    pub id: ParticipantId,
}

pub async fn participant_id_in_use(
    redis_conn: &mut RedisConnection,
    participant_id: ParticipantId,
) -> Result<bool, SignalingModuleError> {
    redis_conn
        .exists(ParticipantIdRunnerLock { id: participant_id })
        .await
        .context(RedisSnafu {
            message: "failed to check if participant id is in use",
        })
}

/// Key used for setting the `skip_waiting_room` attribute for a participant
#[derive(Debug, ToRedisArgs)]
#[to_redis_args(fmt = "opentalk-signaling:participant={participant}:skip_waiting_room")]
pub struct SkipWaitingRoom {
    participant: ParticipantId,
}

/// Set the `skip_waiting_room` key for participant with an expiry.
#[tracing::instrument(level = "debug", skip(redis_conn))]
pub async fn set_skip_waiting_room_with_expiry(
    redis_conn: &mut RedisConnection,
    participant: ParticipantId,
    value: bool,
) -> Result<(), SignalingModuleError> {
    redis_conn
        .set_ex(
            SkipWaitingRoom { participant },
            value,
            SKIP_WAITING_ROOM_KEY_EXPIRY.into(),
        )
        .await
        .with_context(|_| RedisSnafu {
            message: format!(
                "Failed to set skip_waiting_room key to {} for participant {}",
                value, participant,
            ),
        })?;

    Ok(())
}

/// Set the `skip_waiting_room` key for participant with an expiry if the key does not exist.
#[tracing::instrument(level = "debug", skip(redis_conn))]
pub async fn set_skip_waiting_room_with_expiry_nx(
    redis_conn: &mut RedisConnection,
    participant: ParticipantId,
    value: bool,
) -> Result<(), SignalingModuleError> {
    redis::pipe()
        .atomic()
        .set_nx(SkipWaitingRoom { participant }, value)
        .expire(
            SkipWaitingRoom { participant },
            SKIP_WAITING_ROOM_KEY_EXPIRY.into(),
        )
        .query_async(redis_conn)
        .await
        .with_context(|_| RedisSnafu {
            message: format!(
                "Failed to set SkipWaitingRoom key to {} for participant {}",
                value, participant,
            ),
        })?;

    Ok(())
}

/// Extend the `skip_waiting_room` key for participant with an expiry.
#[tracing::instrument(level = "debug", skip(redis_conn))]
pub async fn reset_skip_waiting_room_expiry(
    redis_conn: &mut RedisConnection,
    participant: ParticipantId,
) -> Result<(), SignalingModuleError> {
    redis_conn
        .expire(
            SkipWaitingRoom { participant },
            SKIP_WAITING_ROOM_KEY_EXPIRY.into(),
        )
        .await
        .with_context(|_| RedisSnafu {
            message: format!(
                "Failed to extend skip_waiting_room key expiry for participant {}",
                participant,
            ),
        })?;

    Ok(())
}

/// Get the `skip_waiting_room` value for participant. If no value is set for the key,
/// false is returned.
#[tracing::instrument(level = "debug", skip(redis_conn))]
pub async fn get_skip_waiting_room(
    redis_conn: &mut RedisConnection,
    participant: ParticipantId,
) -> Result<bool, SignalingModuleError> {
    let value: Option<bool> = redis_conn
        .get(SkipWaitingRoom { participant })
        .await
        .context(RedisSnafu {
            message: "Failed to get 'skip waiting room'",
        })?;
    Ok(value.unwrap_or_default())
}

#[tracing::instrument(level = "debug", skip(redis_conn))]
pub async fn get_tariff(
    redis_conn: &mut RedisConnection,
    room_id: RoomId,
) -> Result<Tariff, SignalingModuleError> {
    redis_conn
        .get(RoomTariff { room_id })
        .await
        .context(RedisSnafu {
            message: "Failed to get room tariff",
        })
}

#[tracing::instrument(level = "debug", skip(redis_conn))]
pub async fn delete_tariff(
    redis_conn: &mut RedisConnection,
    room_id: RoomId,
) -> Result<(), SignalingModuleError> {
    redis_conn
        .del(RoomTariff { room_id })
        .await
        .context(RedisSnafu {
            message: "Failed to delete room tariff",
        })
}

/// Try to set the active event for the room. If the event is already set return the current one.
#[tracing::instrument(level = "debug", skip(redis_conn))]
pub async fn try_init_event(
    redis_conn: &mut RedisConnection,
    room_id: RoomId,
    event: Option<Event>,
) -> Result<Option<Event>, SignalingModuleError> {
    let event = if let Some(event) = event {
        let (_, event): (bool, Event) = redis::pipe()
            .atomic()
            .set_nx(RoomEvent { room_id }, event)
            .get(RoomEvent { room_id })
            .query_async(redis_conn)
            .await
            .context(RedisSnafu {
                message: "Failed to SET NX & GET room event",
            })?;

        Some(event)
    } else {
        event
    };

    Ok(event)
}

#[tracing::instrument(level = "debug", skip(redis_conn))]
pub async fn get_event(
    redis_conn: &mut RedisConnection,
    room_id: RoomId,
) -> Result<Option<Event>, SignalingModuleError> {
    redis_conn
        .get(RoomEvent { room_id })
        .await
        .context(RedisSnafu {
            message: "Failed to get room event",
        })
}

#[tracing::instrument(level = "debug", skip(redis_conn))]
pub async fn delete_event(
    redis_conn: &mut RedisConnection,
    room_id: RoomId,
) -> Result<(), SignalingModuleError> {
    redis_conn
        .del(RoomEvent { room_id })
        .await
        .context(RedisSnafu {
            message: "Failed to delete room event",
        })
}

#[tracing::instrument(level = "debug", skip(redis_conn))]
pub async fn increment_participant_count(
    redis_conn: &mut RedisConnection,
    room_id: RoomId,
) -> Result<isize, SignalingModuleError> {
    redis_conn
        .incr(RoomParticipantCount { room_id }, 1)
        .await
        .context(RedisSnafu {
            message: "Failed to increment room participant count",
        })
}

#[tracing::instrument(level = "debug", skip(redis_conn))]
pub async fn decrement_participant_count(
    redis_conn: &mut RedisConnection,
    room_id: RoomId,
) -> Result<isize, SignalingModuleError> {
    redis_conn
        .decr(RoomParticipantCount { room_id }, 1)
        .await
        .context(RedisSnafu {
            message: "Failed to decrement room participant count",
        })
}

#[tracing::instrument(level = "debug", skip(redis_conn))]
pub async fn get_participant_count(
    redis_conn: &mut RedisConnection,
    room_id: RoomId,
) -> Result<Option<isize>, SignalingModuleError> {
    redis_conn
        .get(RoomParticipantCount { room_id })
        .await
        .context(RedisSnafu {
            message: "Failed to get room participant count",
        })
}

#[tracing::instrument(level = "debug", skip(redis_conn))]
pub async fn delete_participant_count(
    redis_conn: &mut RedisConnection,
    room_id: RoomId,
) -> Result<(), SignalingModuleError> {
    redis_conn
        .del(RoomParticipantCount { room_id })
        .await
        .context(RedisSnafu {
            message: "Failed to delete room participant count key",
        })
}

#[tracing::instrument(level = "debug", skip(redis_conn))]
pub async fn set_room_closes_at(
    redis_conn: &mut RedisConnection,
    room: SignalingRoomId,
    timestamp: Timestamp,
) -> Result<(), SignalingModuleError> {
    redis_conn
        .set(RoomClosesAt { room }, timestamp)
        .await
        .context(RedisSnafu {
            message: "Failed to SET the point in time the room closes",
        })
}

#[tracing::instrument(level = "debug", skip(redis_conn))]
pub async fn get_room_closes_at(
    redis_conn: &mut RedisConnection,
    room: SignalingRoomId,
) -> Result<Option<Timestamp>, SignalingModuleError> {
    let key = RoomClosesAt { room };
    redis_conn.get(&key).await.context(RedisSnafu {
        message: "Failed to GET the point in time the room closes",
    })
}

#[tracing::instrument(level = "debug", skip(redis_conn))]
pub async fn remove_room_closes_at(
    redis_conn: &mut RedisConnection,
    room: SignalingRoomId,
) -> Result<(), SignalingModuleError> {
    redis_conn
        .del(RoomClosesAt { room })
        .await
        .context(RedisSnafu {
            message: "Failed to DEL the point in time the room closes",
        })
}

#[cfg(test)]
mod test {
    use redis::aio::ConnectionManager;
    use serial_test::serial;

    use super::{super::test_common, *};

    async fn setup() -> RedisConnection {
        let redis_url =
            std::env::var("REDIS_ADDR").unwrap_or_else(|_| "redis://0.0.0.0:6379/".to_owned());
        let redis = redis::Client::open(redis_url).expect("Invalid redis url");

        let mut mgr = ConnectionManager::new(redis).await.unwrap();

        redis::cmd("FLUSHALL")
            .query_async::<_, ()>(&mut mgr)
            .await
            .unwrap();

        RedisConnection::new(mgr)
    }

    #[tokio::test]
    #[serial]
    async fn participant_set() {
        let mut storage = setup().await;
        test_common::participant_set(&mut storage).await;
    }

    #[tokio::test]
    #[serial]
    async fn participant_attribute() {
        let mut storage = setup().await;
        test_common::participant_attribute(&mut storage).await;
    }

    #[tokio::test]
    #[serial]
    async fn participant_attributes() {
        let mut storage = setup().await;
        test_common::participant_attributes(&mut storage).await;
    }

    #[tokio::test]
    #[serial]
    async fn participant_remove_attributes() {
        let mut storage = setup().await;
        test_common::participant_remove_attributes(&mut storage).await;
    }

    #[tokio::test]
    #[serial]
    async fn get_role_and_left_for_room_participants() {
        let mut storage = setup().await;
        test_common::get_role_and_left_for_room_participants(&mut storage).await;
    }
}
