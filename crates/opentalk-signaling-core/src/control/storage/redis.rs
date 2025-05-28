// SPDX-FileCopyrightText: OpenTalk GmbH <mail@opentalk.eu>
//
// SPDX-License-Identifier: EUPL-1.2

use std::{
    collections::{BTreeMap, BTreeSet, HashMap, HashSet},
    convert::identity,
    fmt::Debug,
};

use async_trait::async_trait;
use opentalk_db_storage::{events::Event, tariffs::Tariff};
use opentalk_types_common::{rooms::RoomId, time::Timestamp, users::UserInfo};
use opentalk_types_signaling::{ParticipantId, Role};
use redis::{AsyncCommands, ErrorKind, FromRedisValue, RedisError, ToRedisArgs};
use redis_args::ToRedisArgs;
use serde::{de::DeserializeOwned, Serialize};
use snafu::ResultExt;

use super::{
    control_storage::{
        AttributeAction, ControlStorageEvent, ControlStorageParticipantSet,
        ControlStorageSkipWaitingRoom, GlobalRoomAttributeId, LocalRoomAttributeId,
        RoomAttributeId,
    },
    AttributeActions, ControlStorage, ControlStorageParticipantAttributesRaw, LEFT_AT, ROLE,
    SKIP_WAITING_ROOM_KEY_EXPIRY,
};
use crate::{RedisConnection, RedisSnafu, SignalingModuleError, SignalingRoomId};

#[async_trait(?Send)]
impl ControlStorage for RedisConnection {
    #[tracing::instrument(level = "debug", skip(self))]
    async fn remove_attribute_key(
        &mut self,
        attribute: RoomAttributeId,
    ) -> Result<(), SignalingModuleError> {
        self.del(attribute).await.with_context(|_| RedisSnafu {
            message: format!("Failed to remove participant attribute key, {attribute}"),
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
        pipe.hgetall(GlobalRoomAttributeId {
            room: room.room_id(),
            attribute: ROLE,
        });
        pipe.hgetall(LocalRoomAttributeId {
            room,
            attribute: LEFT_AT,
        });

        let (mut roles, mut left_at_timestamps): (
            HashMap<ParticipantId, WrappedAttributeValue<Role>>,
            HashMap<ParticipantId, WrappedAttributeValue<Timestamp>>,
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
            .map(|p| {
                (
                    p,
                    (
                        roles.remove(&p).and_then(|v| v.0),
                        left_at_timestamps.remove(&p).and_then(|v| v.0),
                    ),
                )
            })
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

    #[tracing::instrument(level = "debug", skip(self))]
    async fn get_tariff(&mut self, room_id: RoomId) -> Result<Tariff, SignalingModuleError> {
        self.get(RoomTariff { room_id }).await.context(RedisSnafu {
            message: "Failed to get room tariff",
        })
    }

    #[tracing::instrument(level = "debug", skip(self))]
    async fn delete_tariff(&mut self, room_id: RoomId) -> Result<(), SignalingModuleError> {
        self.del(RoomTariff { room_id }).await.context(RedisSnafu {
            message: "Failed to delete room tariff",
        })
    }

    #[tracing::instrument(level = "debug", skip(self))]
    async fn increment_participant_count(
        &mut self,
        room_id: RoomId,
    ) -> Result<isize, SignalingModuleError> {
        self.incr(RoomParticipantCount { room_id }, 1)
            .await
            .context(RedisSnafu {
                message: "Failed to increment room participant count",
            })
    }

    #[tracing::instrument(level = "debug", skip(self))]
    async fn decrement_participant_count(
        &mut self,
        room_id: RoomId,
    ) -> Result<isize, SignalingModuleError> {
        self.decr(RoomParticipantCount { room_id }, 1)
            .await
            .context(RedisSnafu {
                message: "Failed to decrement room participant count",
            })
    }

    #[tracing::instrument(level = "debug", skip(self))]
    async fn get_participant_count(
        &mut self,
        room_id: RoomId,
    ) -> Result<Option<isize>, SignalingModuleError> {
        self.get(RoomParticipantCount { room_id })
            .await
            .context(RedisSnafu {
                message: "Failed to get room participant count",
            })
    }

    #[tracing::instrument(level = "debug", skip(self))]
    async fn delete_participant_count(
        &mut self,
        room_id: RoomId,
    ) -> Result<(), SignalingModuleError> {
        self.del(RoomParticipantCount { room_id })
            .await
            .context(RedisSnafu {
                message: "Failed to delete room participant count key",
            })
    }

    #[tracing::instrument(level = "debug", skip(self))]
    async fn try_init_creator(
        &mut self,
        room_id: RoomId,
        creator: UserInfo,
    ) -> Result<UserInfo, SignalingModuleError> {
        let (_, creator): (bool, UserInfo) = redis::pipe()
            .atomic()
            .set_nx(RoomCreator { room_id }, creator)
            .get(RoomCreator { room_id })
            .query_async(self)
            .await
            .context(RedisSnafu {
                message: "Failed to SET NX & GET room creator",
            })?;

        Ok(creator)
    }

    #[tracing::instrument(level = "debug", skip(self))]
    async fn get_creator(
        &mut self,
        room_id: RoomId,
    ) -> Result<Option<UserInfo>, SignalingModuleError> {
        self.get(RoomCreator { room_id }).await.context(RedisSnafu {
            message: "Failed to get room creator",
        })
    }

    #[tracing::instrument(level = "debug", skip(self))]
    async fn delete_creator(&mut self, room_id: RoomId) -> Result<(), SignalingModuleError> {
        self.del(RoomCreator { room_id }).await.context(RedisSnafu {
            message: "Failed to delete room creator",
        })
    }

    #[tracing::instrument(level = "debug", skip(self))]
    async fn set_room_closes_at(
        &mut self,
        room: SignalingRoomId,
        timestamp: Timestamp,
    ) -> Result<(), SignalingModuleError> {
        self.set(RoomClosesAt { room }, timestamp)
            .await
            .context(RedisSnafu {
                message: "Failed to SET the point in time the room closes",
            })
    }

    #[tracing::instrument(level = "debug", skip(self))]
    async fn get_room_closes_at(
        &mut self,
        room: SignalingRoomId,
    ) -> Result<Option<Timestamp>, SignalingModuleError> {
        let key = RoomClosesAt { room };
        self.get(&key).await.context(RedisSnafu {
            message: "Failed to GET the point in time the room closes",
        })
    }

    #[tracing::instrument(level = "debug", skip(self))]
    async fn remove_room_closes_at(
        &mut self,
        room: SignalingRoomId,
    ) -> Result<(), SignalingModuleError> {
        self.del(RoomClosesAt { room }).await.context(RedisSnafu {
            message: "Failed to DEL the point in time the room closes",
        })
    }

    #[tracing::instrument(level = "debug", skip(self))]
    async fn set_room_alive(&mut self, room: RoomId) -> Result<(), SignalingModuleError> {
        self.set(RoomAlive { room }, true)
            .await
            .context(RedisSnafu {
                message: "Failed to SET room alive key",
            })
    }

    #[tracing::instrument(level = "debug", skip(self))]
    async fn is_room_alive(&mut self, room: RoomId) -> Result<bool, SignalingModuleError> {
        self.exists(RoomAlive { room }).await.context(RedisSnafu {
            message: "Failed to check if room alive key exists",
        })
    }

    #[tracing::instrument(level = "debug", skip(self))]
    async fn delete_room_alive(&mut self, room: RoomId) -> Result<(), SignalingModuleError> {
        self.del(RoomAlive { room }).await.context(RedisSnafu {
            message: "Failed to DEL room alive key",
        })
    }
}

#[async_trait(?Send)]
impl ControlStorageSkipWaitingRoom for RedisConnection {
    #[tracing::instrument(level = "debug", skip(self))]
    async fn set_skip_waiting_room_with_expiry(
        &mut self,
        participant: ParticipantId,
        value: bool,
    ) -> Result<(), SignalingModuleError> {
        self.set_ex::<_, _, ()>(
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

    #[tracing::instrument(level = "debug", skip(self))]
    async fn set_skip_waiting_room_with_expiry_nx(
        &mut self,
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
            .exec_async(self)
            .await
            .with_context(|_| RedisSnafu {
                message: format!(
                    "Failed to set SkipWaitingRoom key to {} for participant {}",
                    value, participant,
                ),
            })?;

        Ok(())
    }

    #[tracing::instrument(level = "debug", skip(self))]
    async fn reset_skip_waiting_room_expiry(
        &mut self,
        participant: ParticipantId,
    ) -> Result<(), SignalingModuleError> {
        self.expire::<_, ()>(
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

    #[tracing::instrument(level = "debug", skip(self))]
    async fn get_skip_waiting_room(
        &mut self,
        participant: ParticipantId,
    ) -> Result<bool, SignalingModuleError> {
        let value: Option<bool> =
            self.get(SkipWaitingRoom { participant })
                .await
                .context(RedisSnafu {
                    message: "Failed to get 'skip waiting room'",
                })?;
        Ok(value.unwrap_or_default())
    }
}

#[async_trait(?Send)]
impl ControlStorageParticipantSet for RedisConnection {
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
}

/// Describes a set of participants inside a room.
/// This MUST always be locked before accessing it
#[derive(ToRedisArgs)]
#[to_redis_args(fmt = "opentalk-signaling:room={room}:participants")]
struct RoomParticipants {
    room: SignalingRoomId,
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

/// The point in time the room closes.
#[derive(ToRedisArgs)]
#[to_redis_args(fmt = "opentalk-signaling:room={room}:closes_at")]
struct RoomClosesAt {
    room: SignalingRoomId,
}

/// The key to check if the room was cleaned up
#[derive(ToRedisArgs)]
#[to_redis_args(fmt = "opentalk-signaling:room={room}:room_alive")]
struct RoomAlive {
    room: RoomId,
}

/// Key used for setting the `skip_waiting_room` attribute for a participant
#[derive(Debug, ToRedisArgs)]
#[to_redis_args(fmt = "opentalk-signaling:participant={participant}:skip_waiting_room")]
pub struct SkipWaitingRoom {
    participant: ParticipantId,
}

struct WrappedAttributeValueJson(Option<serde_json::Value>);

impl ToRedisArgs for WrappedAttributeValueJson {
    fn write_redis_args<W>(&self, out: &mut W)
    where
        W: ?Sized + redis::RedisWrite,
    {
        let value =
            serde_json::to_vec(&self.0).expect("serde_json::Value should always be serializable");
        out.write_arg(&value)
    }
}

impl FromRedisValue for WrappedAttributeValueJson {
    fn from_redis_value(v: &redis::Value) -> redis::RedisResult<Self> {
        match v {
            redis::Value::Nil => Ok(Self(None)),
            redis::Value::Int(v) => Ok(Self(Some(serde_json::Value::Number(
                serde_json::Number::from(*v),
            )))),
            redis::Value::BulkString(v) => {
                let value = serde_json::from_slice(v).map_err(|e| {
                    redis::RedisError::from((
                        redis::ErrorKind::ParseError,
                        "Could not deserialize JSON value",
                        format!("{:?}", e),
                    ))
                })?;
                Ok(Self(Some(value)))
            }
            redis::Value::Array(v) | redis::Value::Set(v) => {
                let values = v
                    .iter()
                    .map(WrappedAttributeValueJson::from_redis_value)
                    .collect::<redis::RedisResult<Vec<WrappedAttributeValueJson>>>()?;
                let values: Vec<serde_json::Value> = values
                    .into_iter()
                    .map(|v| serde_json::to_value(v.0).expect("Option<Value> must be serializable"))
                    .collect();
                Ok(Self(Some(serde_json::Value::Array(values))))
            }
            v @ (redis::Value::SimpleString(_)
            | redis::Value::Okay
            | redis::Value::ServerError(_)
            | redis::Value::Map(_)
            | redis::Value::Double(_)
            | redis::Value::Boolean(_)
            | redis::Value::BigNumber(_)
            | redis::Value::VerbatimString { .. }
            | redis::Value::Attribute { .. }
            | redis::Value::Push { .. }) => Err(redis::RedisError::from((
                redis::ErrorKind::TypeError,
                "Response was of incompatible type",
                format!("response was {:?}", v),
            ))),
        }
    }
}

struct WrappedAttributeValue<T>(Option<T>);

impl<T: Serialize> ToRedisArgs for WrappedAttributeValue<T> {
    fn write_redis_args<W>(&self, out: &mut W)
    where
        W: ?Sized + redis::RedisWrite,
    {
        let value = serde_json::to_vec(&self.0).expect("value must be serializable");
        out.write_arg(&value)
    }
}

impl<T: DeserializeOwned> FromRedisValue for WrappedAttributeValue<T> {
    fn from_redis_value(v: &redis::Value) -> redis::RedisResult<Self> {
        let WrappedAttributeValueJson(value) = WrappedAttributeValueJson::from_redis_value(v)?;
        let Some(value) = value else {
            return Ok(Self(None));
        };

        let value = serde_json::from_value(value).map_err(|e| {
            redis::RedisError::from((
                redis::ErrorKind::ParseError,
                "Could not deserialize JSON value",
                format!("{:?}", e),
            ))
        })?;
        Ok(Self(Some(value)))
    }
}

#[async_trait(?Send)]
impl ControlStorageParticipantAttributesRaw for RedisConnection {
    #[tracing::instrument(level = "debug", skip(self))]
    async fn get_attribute_raw(
        &mut self,
        participant: ParticipantId,
        attribute: RoomAttributeId,
    ) -> Result<Option<serde_json::Value>, SignalingModuleError> {
        let WrappedAttributeValueJson(value) = self
            .hget(attribute, participant)
            .await
            .with_context(|_| RedisSnafu {
                message: format!("Failed to get attribute {attribute}"),
            })?;

        Ok(value)
    }

    #[tracing::instrument(level = "debug", skip(self))]
    async fn get_attribute_for_participants_raw(
        &mut self,
        participants: &[ParticipantId],
        attribute: RoomAttributeId,
    ) -> Result<Vec<Option<serde_json::Value>>, SignalingModuleError> {
        // Special case: HMGET cannot handle empty arrays (missing arguments)
        if participants.is_empty() {
            Ok(vec![])
        } else {
            // need manual HMGET command as the HGET command wont work with single value vector input
            let WrappedAttributeValue::<Vec<Option<serde_json::Value>>>(value) =
                redis::cmd("HMGET")
                    .arg(attribute)
                    .arg(participants)
                    .query_async(self)
                    .await
                    .with_context(|_| RedisSnafu {
                        message: format!(
                            "Failed to get attribute '{attribute}' for all participants"
                        ),
                    })?;
            Ok(value.unwrap_or_default())
        }
    }

    #[tracing::instrument(level = "debug", skip(self))]
    async fn set_attribute_raw(
        &mut self,
        participant: ParticipantId,
        attribute: RoomAttributeId,
        value: serde_json::Value,
    ) -> Result<(), SignalingModuleError> {
        self.hset::<_, _, _, ()>(
            attribute,
            participant,
            WrappedAttributeValueJson(Some(value)),
        )
        .await
        .with_context(|_| RedisSnafu {
            message: format!("Failed to set attribute {attribute}"),
        })?;

        Ok(())
    }

    #[tracing::instrument(level = "debug", skip(self))]
    async fn remove_attribute_raw(
        &mut self,
        participant: ParticipantId,
        attribute: RoomAttributeId,
    ) -> Result<(), SignalingModuleError> {
        self.hdel(attribute, participant)
            .await
            .with_context(|_| RedisSnafu {
                message: format!("Failed to remove participant attribute key, {attribute}"),
            })
    }

    #[tracing::instrument(level = "debug", skip(self, actions))]
    async fn bulk_attribute_actions_raw(
        &mut self,
        actions: &AttributeActions,
    ) -> Result<serde_json::Value, SignalingModuleError> {
        let participant = actions.participant();

        let mut pipe = redis::pipe();
        pipe.atomic();
        for action in actions.actions() {
            match action {
                AttributeAction::Set { attribute, value } => {
                    pipe.hset(
                        attribute,
                        participant,
                        WrappedAttributeValueJson(Some(value.clone())),
                    )
                    .ignore();
                }
                AttributeAction::Get { attribute } => {
                    pipe.hget(attribute, participant);
                }
                AttributeAction::Delete { attribute } => {
                    pipe.hdel(attribute, participant).ignore();
                }
            }
        }

        let WrappedAttributeValueJson(Some(mut value)) =
            pipe.query_async(self).await.with_context(|_| RedisSnafu {
                message: "Failed to perform bulk attribute actions".to_string(),
            })?
        else {
            return Err(RedisError::from((ErrorKind::TypeError, "Empty value"))).context(
                RedisSnafu {
                    message: "Redis bulk action error",
                },
            );
        };
        if value == serde_json::Value::Array(Vec::new()) {
            value = serde_json::Value::Null;
        }
        Ok(value)
    }
}

#[async_trait(?Send)]
impl ControlStorageEvent for RedisConnection {
    #[tracing::instrument(level = "debug", skip(self))]
    async fn try_init_event(
        &mut self,
        room_id: RoomId,
        event: Option<Event>,
    ) -> Result<Option<Event>, SignalingModuleError> {
        let event = if let Some(event) = event {
            let (_, event): (bool, Event) = redis::pipe()
                .atomic()
                .set_nx(RoomEvent { room_id }, event)
                .get(RoomEvent { room_id })
                .query_async(self)
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

    #[tracing::instrument(level = "debug", skip(self))]
    async fn get_event(&mut self, room_id: RoomId) -> Result<Option<Event>, SignalingModuleError> {
        self.get(RoomEvent { room_id }).await.context(RedisSnafu {
            message: "Failed to get room event",
        })
    }

    #[tracing::instrument(level = "debug", skip(self))]
    async fn delete_event(&mut self, room_id: RoomId) -> Result<(), SignalingModuleError> {
        self.del(RoomEvent { room_id }).await.context(RedisSnafu {
            message: "Failed to delete room event",
        })
    }
}

/// The associated [`Event`] for the room
///
/// Notice that this key only contains the [`RoomId`] as it applies to all breakout rooms as well
#[derive(ToRedisArgs)]
#[to_redis_args(fmt = "opentalk-signaling:room={room_id}:event")]
pub struct RoomEvent {
    room_id: RoomId,
}

/// The [`CreatorInfo`]  for the user that created the room
///
/// Notice that this key only contains the [`RoomId`] as it applies to all breakout rooms as well
#[derive(ToRedisArgs)]
#[to_redis_args(fmt = "opentalk-signaling:room={room_id}:creator")]
pub struct RoomCreator {
    room_id: RoomId,
}

#[cfg(test)]
mod tests {
    use redis::aio::ConnectionManager;
    use serial_test::serial;

    use super::{super::test_common, *};

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
    async fn participant_set() {
        test_common::participant_set(&mut storage().await).await;
    }

    #[tokio::test]
    #[serial]
    async fn participant_attribute() {
        test_common::participant_attribute(&mut storage().await).await;
    }

    #[tokio::test]
    #[serial]
    async fn participant_attribute_empty() {
        test_common::participant_attribute_empty(&mut storage().await).await;
    }

    #[tokio::test]
    #[serial]
    async fn participant_attributes() {
        test_common::participant_attributes(&mut storage().await).await;
    }

    #[tokio::test]
    #[serial]
    async fn participant_remove_attributes() {
        test_common::participant_remove_attributes(&mut storage().await).await;
    }

    #[tokio::test]
    #[serial]
    async fn get_role_and_left_for_room_participants() {
        test_common::get_role_and_left_for_room_participants(&mut storage().await).await;
    }

    #[tokio::test]
    #[serial]
    async fn participant_attributes_bulk() {
        test_common::participant_attributes_bulk(&mut storage().await).await;
    }

    #[tokio::test]
    #[serial]
    async fn tariff() {
        test_common::tariff(&mut storage().await).await;
    }

    #[tokio::test]
    #[serial]
    async fn event() {
        test_common::event(&mut storage().await).await;
    }

    #[tokio::test]
    #[serial]
    async fn participant_count() {
        test_common::participant_count(&mut storage().await).await;
    }

    #[tokio::test]
    #[serial]
    async fn creator_info() {
        test_common::creator_info(&mut storage().await).await;
    }

    #[tokio::test]
    #[serial]
    async fn room_closes_at() {
        test_common::room_closes_at(&mut storage().await).await;
    }

    #[tokio::test]
    #[serial]
    async fn skip_waiting_room() {
        test_common::skip_waiting_room(&mut storage().await).await;
    }
}
