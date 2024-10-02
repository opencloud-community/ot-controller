// SPDX-FileCopyrightText: OpenTalk GmbH <mail@opentalk.eu>
//
// SPDX-License-Identifier: EUPL-1.2

use async_trait::async_trait;
use opentalk_signaling_core::{RedisConnection, RedisSnafu, SignalingModuleError, SignalingRoomId};
use opentalk_types::signaling::timer::ready_status::ReadyStatus;
use opentalk_types_signaling::ParticipantId;
use redis::AsyncCommands;
use redis_args::ToRedisArgs;
use snafu::ResultExt;

use super::{Timer, TimerStorage};

#[async_trait(?Send)]
impl TimerStorage for RedisConnection {
    #[tracing::instrument(name = "meeting_timer_ready_set", skip(self))]
    async fn ready_status_set(
        &mut self,
        room_id: SignalingRoomId,
        participant_id: ParticipantId,
        ready_status: bool,
    ) -> Result<(), SignalingModuleError> {
        self.set(
            ReadyStatusKey {
                room_id,
                participant_id,
            },
            &ReadyStatus { ready_status },
        )
        .await
        .context(RedisSnafu {
            message: "Failed to set ready state",
        })
    }

    #[tracing::instrument(name = "meeting_timer_ready_get", skip(self))]
    async fn ready_status_get(
        &mut self,
        room_id: SignalingRoomId,
        participant_id: ParticipantId,
    ) -> Result<Option<ReadyStatus>, SignalingModuleError> {
        self.get(ReadyStatusKey {
            room_id,
            participant_id,
        })
        .await
        .context(RedisSnafu {
            message: "Failed to get ready state",
        })
    }

    #[tracing::instrument(name = "meeting_timer_ready_delete", skip(self))]
    async fn ready_status_delete(
        &mut self,
        room_id: SignalingRoomId,
        participant_id: ParticipantId,
    ) -> Result<(), SignalingModuleError> {
        self.del(ReadyStatusKey {
            room_id,
            participant_id,
        })
        .await
        .context(RedisSnafu {
            message: "Failed to delete ready state",
        })
    }

    #[tracing::instrument(name = "meeting_timer_set", skip(self, timer))]
    async fn timer_set_if_not_exists(
        &mut self,
        room_id: SignalingRoomId,
        timer: &Timer,
    ) -> Result<bool, SignalingModuleError> {
        self.set_nx(TimerKey { room_id }, timer)
            .await
            .context(RedisSnafu {
                message: "Failed to set meeting timer",
            })
    }

    #[tracing::instrument(name = "meeting_timer_get", skip(self))]
    async fn timer_get(
        &mut self,
        room_id: SignalingRoomId,
    ) -> Result<Option<Timer>, SignalingModuleError> {
        self.get(TimerKey { room_id }).await.context(RedisSnafu {
            message: "Failed to get meeting timer",
        })
    }

    /// Delete the current timer
    ///
    /// Returns the timer if there was any
    #[tracing::instrument(name = "meeting_timer_delete", skip(self))]
    async fn timer_delete(
        &mut self,
        room_id: SignalingRoomId,
    ) -> Result<Option<Timer>, SignalingModuleError> {
        redis::cmd("GETDEL")
            .arg(TimerKey { room_id })
            .query_async(self)
            .await
            .context(RedisSnafu {
                message: "Failed to delete meeting timer",
            })
    }
}

/// A key to track the participants ready status
#[derive(ToRedisArgs)]
#[to_redis_args(
    fmt = "opentalk-signaling:room={room_id}:participant::{participant_id}::timer-ready-status"
)]
struct ReadyStatusKey {
    room_id: SignalingRoomId,
    participant_id: ParticipantId,
}

/// The timer key holds a serialized [`Timer`].
#[derive(ToRedisArgs)]
#[to_redis_args(fmt = "opentalk-signaling:room={room_id}:timer")]
struct TimerKey {
    room_id: SignalingRoomId,
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
    async fn ready_status() {
        test_common::ready_status(&mut storage().await).await;
    }
}
