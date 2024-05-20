// SPDX-FileCopyrightText: OpenTalk GmbH <mail@opentalk.eu>
//
// SPDX-License-Identifier: EUPL-1.2

use std::collections::BTreeSet;

use async_trait::async_trait;
use opentalk_signaling_core::{RedisConnection, RedisSnafu, SignalingModuleError};
use opentalk_types::core::{ParticipantId, RoomId, UserId};
use redis::AsyncCommands;
use redis_args::ToRedisArgs;
use snafu::ResultExt;

use super::ModerationStorage;

#[async_trait(?Send)]
impl ModerationStorage for RedisConnection {
    #[tracing::instrument(level = "debug", skip(self))]
    async fn ban_user(&mut self, room: RoomId, user: UserId) -> Result<(), SignalingModuleError> {
        self.sadd(Bans { room }, user).await.context(RedisSnafu {
            message: "Failed to SADD user_id to bans",
        })
    }

    #[tracing::instrument(level = "debug", skip(self))]
    async fn is_user_banned(
        &mut self,
        room: RoomId,
        user_id: UserId,
    ) -> Result<bool, SignalingModuleError> {
        self.sismember(Bans { room }, user_id)
            .await
            .context(RedisSnafu {
                message: "Failed to SISMEMBER user_id on bans",
            })
    }

    #[tracing::instrument(level = "debug", skip(self))]
    async fn delete_user_bans(&mut self, room: RoomId) -> Result<(), SignalingModuleError> {
        self.del(Bans { room }).await.context(RedisSnafu {
            message: "Failed to DEL bans",
        })
    }

    #[tracing::instrument(level = "debug", skip(self))]
    async fn init_waiting_room_enabled(
        &mut self,
        room: RoomId,
        enabled: bool,
    ) -> Result<bool, SignalingModuleError> {
        let was_enabled: (bool, bool) = redis::pipe()
            .atomic()
            .set_nx(WaitingRoomEnabled { room }, enabled)
            .get(WaitingRoomEnabled { room })
            .query_async(self)
            .await
            .context(RedisSnafu {
                message: "Failed SET-or-GET waiting_room_enabled",
            })?;
        Ok(was_enabled.1)
    }

    #[tracing::instrument(level = "debug", skip(self))]
    async fn set_waiting_room_enabled(
        &mut self,
        room: RoomId,
        enabled: bool,
    ) -> Result<(), SignalingModuleError> {
        self.set(WaitingRoomEnabled { room }, enabled)
            .await
            .context(RedisSnafu {
                message: "Failed to SET waiting_room_enabled",
            })
    }

    #[tracing::instrument(level = "debug", skip(self))]
    async fn is_waiting_room_enabled(
        &mut self,
        room: RoomId,
    ) -> Result<bool, SignalingModuleError> {
        self.get(WaitingRoomEnabled { room })
            .await
            .context(RedisSnafu {
                message: "Failed to GET waiting_room_enabled",
            })
            .map(Option::<bool>::unwrap_or_default)
    }

    #[tracing::instrument(level = "debug", skip(self))]
    async fn delete_waiting_room_enabled(
        &mut self,
        room: RoomId,
    ) -> Result<(), SignalingModuleError> {
        self.del(WaitingRoomEnabled { room })
            .await
            .context(RedisSnafu {
                message: "Failed to DEL waiting_room_enabled",
            })
    }

    #[tracing::instrument(level = "debug", skip(self))]
    async fn set_raise_hands_enabled(
        &mut self,
        room: RoomId,
        enabled: bool,
    ) -> Result<(), SignalingModuleError> {
        self.set(RaiseHandsEnabled { room }, enabled)
            .await
            .context(RedisSnafu {
                message: "Failed to SET raise_hands_enabled",
            })
    }

    #[tracing::instrument(level = "debug", skip(self))]
    async fn is_raise_hands_enabled(&mut self, room: RoomId) -> Result<bool, SignalingModuleError> {
        self.get(RaiseHandsEnabled { room })
            .await
            .context(RedisSnafu {
                message: "Failed to GET raise_hands_enabled",
            })
            .map(|result: Option<bool>| result.unwrap_or(true))
    }

    #[tracing::instrument(level = "debug", skip(self))]
    async fn delete_raise_hands_enabled(
        &mut self,
        room: RoomId,
    ) -> Result<(), SignalingModuleError> {
        self.del(RaiseHandsEnabled { room })
            .await
            .context(RedisSnafu {
                message: "Failed to DEL raise_hands_enabled",
            })
    }

    #[tracing::instrument(level = "debug", skip(self))]
    async fn waiting_room_add_participant(
        &mut self,
        room: RoomId,
        participant: ParticipantId,
    ) -> Result<bool, SignalingModuleError> {
        self.sadd::<_, _, usize>(WaitingRoomList { room }, participant)
            .await
            .context(RedisSnafu {
                message: "Failed to SADD waiting_room_list",
            })
            .map(|count| count > 0)
    }

    #[tracing::instrument(level = "debug", skip(self))]
    async fn waiting_room_remove_participant(
        &mut self,
        room: RoomId,
        participant: ParticipantId,
    ) -> Result<(), SignalingModuleError> {
        self.srem(WaitingRoomList { room }, participant)
            .await
            .context(RedisSnafu {
                message: "Failed to SREM waiting_room_list",
            })
    }

    #[tracing::instrument(level = "debug", skip(self))]
    async fn waiting_room_contains_participant(
        &mut self,
        room: RoomId,
        participant: ParticipantId,
    ) -> Result<bool, SignalingModuleError> {
        self.sismember(WaitingRoomList { room }, participant)
            .await
            .context(RedisSnafu {
                message: "Failed to SISMEMBER waiting_room_list",
            })
    }

    #[tracing::instrument(level = "debug", skip(self))]
    async fn waiting_room_participants(
        &mut self,
        room: RoomId,
    ) -> Result<BTreeSet<ParticipantId>, SignalingModuleError> {
        self.smembers(WaitingRoomList { room })
            .await
            .context(RedisSnafu {
                message: "Failed to SMEMBERS waiting_room_list",
            })
    }

    #[tracing::instrument(level = "debug", skip(self))]
    async fn waiting_room_participant_count(
        &mut self,
        room: RoomId,
    ) -> Result<usize, SignalingModuleError> {
        self.scard(WaitingRoomList { room })
            .await
            .context(RedisSnafu {
                message: "Failed to SCARD waiting_room_list",
            })
    }

    #[tracing::instrument(level = "debug", skip(self))]
    async fn delete_waiting_room(&mut self, room: RoomId) -> Result<(), SignalingModuleError> {
        self.del(WaitingRoomList { room })
            .await
            .context(RedisSnafu {
                message: "Failed to DEL waiting_room_list",
            })
    }

    #[tracing::instrument(level = "debug", skip(self))]
    async fn waiting_room_accepted_add_participant(
        &mut self,
        room: RoomId,
        participant: ParticipantId,
    ) -> Result<bool, SignalingModuleError> {
        self.sadd(AcceptedWaitingRoomList { room }, participant)
            .await
            .context(RedisSnafu {
                message: "Failed to SADD waiting_room_accepted_list",
            })
    }

    #[tracing::instrument(level = "debug", skip(self))]
    async fn waiting_room_accepted_remove_participant(
        &mut self,
        room: RoomId,
        participant: ParticipantId,
    ) -> Result<(), SignalingModuleError> {
        self.srem(AcceptedWaitingRoomList { room }, participant)
            .await
            .context(RedisSnafu {
                message: "Failed to SREM individual participant from waiting_room_accepted_list",
            })
    }

    #[tracing::instrument(level = "debug", skip(self))]
    async fn waiting_room_accepted_remove_participants(
        &mut self,
        room: RoomId,
        participants: &[ParticipantId],
    ) -> Result<(), SignalingModuleError> {
        if participants.is_empty() {
            return Ok(());
        }

        self.srem(AcceptedWaitingRoomList { room }, participants)
            .await
            .context(RedisSnafu {
                message: "Failed to SREM multiple participants from waiting_room_accepted_list",
            })
    }

    #[tracing::instrument(level = "debug", skip(self))]
    async fn waiting_room_accepted_participants(
        &mut self,
        room: RoomId,
    ) -> Result<BTreeSet<ParticipantId>, SignalingModuleError> {
        self.smembers(AcceptedWaitingRoomList { room })
            .await
            .context(RedisSnafu {
                message: "Failed to SMEMBERS waiting_room_accepted_list",
            })
    }

    #[tracing::instrument(level = "debug", skip(self))]
    async fn waiting_room_accepted_participant_count(
        &mut self,
        room: RoomId,
    ) -> Result<usize, SignalingModuleError> {
        self.scard(AcceptedWaitingRoomList { room })
            .await
            .context(RedisSnafu {
                message: "Failed to SCARD waiting_room_accepted_list",
            })
    }
}

/// Set of user-ids banned in a room
#[derive(ToRedisArgs)]
#[to_redis_args(fmt = "opentalk-signaling:room={room}:bans")]
struct Bans {
    room: RoomId,
}

/// If set to true the waiting room is enabled
#[derive(ToRedisArgs)]
#[to_redis_args(fmt = "opentalk-signaling:room={room}:waiting_room_enabled")]
struct WaitingRoomEnabled {
    room: RoomId,
}

/// If set to true the raise hands is enabled
#[derive(ToRedisArgs)]
#[to_redis_args(fmt = "opentalk-signaling:room={room}:raise_hands_enabled")]
struct RaiseHandsEnabled {
    room: RoomId,
}

/// Set of participant ids inside the waiting room
#[derive(ToRedisArgs)]
#[to_redis_args(fmt = "opentalk-signaling:room={room}:waiting_room_list")]
struct WaitingRoomList {
    room: RoomId,
}

/// Set of participant ids inside the waiting room but accepted
#[derive(ToRedisArgs)]
#[to_redis_args(fmt = "opentalk-signaling:room={room}:waiting_room_accepted_list")]
struct AcceptedWaitingRoomList {
    room: RoomId,
}

#[tracing::instrument(level = "debug", skip(redis_conn))]
pub async fn delete_waiting_room_accepted(
    redis_conn: &mut RedisConnection,
    room: RoomId,
) -> Result<(), SignalingModuleError> {
    redis_conn
        .del(AcceptedWaitingRoomList { room })
        .await
        .context(RedisSnafu {
            message: "Failed to DEL waiting_room_accepted_list",
        })
}

#[cfg(test)]
mod test {
    use redis::aio::ConnectionManager;
    use serial_test::serial;

    use super::{super::test_common, *};

    async fn storage() -> RedisConnection {
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
    async fn user_bans() {
        test_common::user_bans(&mut storage().await).await;
    }

    #[tokio::test]
    #[serial]
    async fn waiting_room_enabled_flag() {
        test_common::waiting_room_enabled_flag(&mut storage().await).await;
    }

    #[tokio::test]
    #[serial]
    async fn raise_hands_enabled_flag() {
        test_common::raise_hands_enabled_flag(&mut storage().await).await;
    }

    #[tokio::test]
    #[serial]
    async fn waiting_room_participants() {
        test_common::waiting_room_participants(&mut storage().await).await;
    }

    #[tokio::test]
    #[serial]
    async fn waiting_room_accepted_participants() {
        test_common::waiting_room_accepted_participants(&mut storage().await).await;
    }
}
