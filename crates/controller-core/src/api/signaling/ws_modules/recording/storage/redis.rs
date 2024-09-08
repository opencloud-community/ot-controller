// SPDX-FileCopyrightText: OpenTalk GmbH <mail@opentalk.eu>
//
// SPDX-License-Identifier: EUPL-1.2

use std::collections::BTreeMap;

use async_trait::async_trait;
use opentalk_signaling_core::{RedisConnection, RedisSnafu, SignalingModuleError, SignalingRoomId};
use opentalk_types_common::streaming::StreamingTargetId;
use opentalk_types_signaling_recording::StreamTargetSecret;
use redis::AsyncCommands;
use redis_args::ToRedisArgs;
use snafu::ResultExt;

use super::RecordingStorage;

#[async_trait(?Send)]
impl RecordingStorage for RedisConnection {
    #[tracing::instrument(level = "debug", skip(self))]
    async fn is_streaming_initialized(
        &mut self,
        room: SignalingRoomId,
    ) -> Result<bool, SignalingModuleError> {
        self.exists(RecordingStreamsKey { room })
            .await
            .context(RedisSnafu {
                message: "Failed to initialize streaming",
            })
    }

    #[tracing::instrument(level = "debug", skip(self))]
    async fn set_streams(
        &mut self,
        room: SignalingRoomId,
        target_streams: &BTreeMap<StreamingTargetId, StreamTargetSecret>,
    ) -> Result<(), SignalingModuleError> {
        let target_streams: Vec<_> = target_streams.iter().collect();
        self.hset_multiple(RecordingStreamsKey { room }, target_streams.as_slice())
            .await
            .context(RedisSnafu {
                message: "Failed to set target stream ids",
            })
    }

    #[tracing::instrument(level = "debug", skip(self))]
    async fn set_stream(
        &mut self,
        room: SignalingRoomId,
        target: StreamingTargetId,
        stream_target: StreamTargetSecret,
    ) -> Result<(), SignalingModuleError> {
        self.hset(RecordingStreamsKey { room }, target, stream_target)
            .await
            .context(RedisSnafu {
                message: "Failed to set target stream",
            })
    }

    #[tracing::instrument(level = "debug", skip(self))]
    async fn get_streams(
        &mut self,
        room: SignalingRoomId,
    ) -> Result<BTreeMap<StreamingTargetId, StreamTargetSecret>, SignalingModuleError> {
        self.hgetall(RecordingStreamsKey { room })
            .await
            .context(RedisSnafu {
                message: "Failed to get all streams",
            })
    }

    #[tracing::instrument(level = "debug", skip(self))]
    async fn get_stream(
        &mut self,
        room: SignalingRoomId,
        target: StreamingTargetId,
    ) -> Result<StreamTargetSecret, SignalingModuleError> {
        self.hget(RecordingStreamsKey { room }, target)
            .await
            .context(RedisSnafu {
                message: "Failed to get target stream",
            })
    }

    #[tracing::instrument(level = "debug", skip(self))]
    async fn stream_exists(
        &mut self,
        room: SignalingRoomId,
        target: StreamingTargetId,
    ) -> Result<bool, SignalingModuleError> {
        self.hexists(RecordingStreamsKey { room }, target)
            .await
            .context(RedisSnafu {
                message: "Failed to check for presence of stream",
            })
    }

    #[tracing::instrument(level = "debug", skip(self))]
    async fn delete_all_streams(
        &mut self,
        room: SignalingRoomId,
    ) -> Result<(), SignalingModuleError> {
        self.del(RecordingStreamsKey { room })
            .await
            .context(RedisSnafu {
                message: "Failed to delete recording state",
            })
    }
}

/// Stores the [`RecordingStatus`] of this room.
#[derive(ToRedisArgs)]
#[to_redis_args(fmt = "opentalk-signaling:room={room}:recording:streams")]
struct RecordingStreamsKey {
    room: SignalingRoomId,
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
    async fn streams() {
        test_common::streams(&mut storage().await).await;
    }

    #[tokio::test]
    #[serial]
    async fn streams_contain_status() {
        test_common::streams_contain_status(&mut storage().await).await;
    }

    #[tokio::test]
    #[serial]
    async fn update_streams_status() {
        test_common::update_streams_status(&mut storage().await).await;
    }
}
