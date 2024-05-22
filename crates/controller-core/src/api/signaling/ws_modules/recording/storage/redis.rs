// SPDX-FileCopyrightText: OpenTalk GmbH <mail@opentalk.eu>
//
// SPDX-License-Identifier: EUPL-1.2

use std::collections::{BTreeMap, BTreeSet};

use async_trait::async_trait;
use itertools::Itertools;
use opentalk_signaling_core::{RedisConnection, RedisSnafu, SignalingModuleError, SignalingRoomId};
use opentalk_types::{
    core::StreamingTargetId,
    signaling::recording::{StreamStatus, StreamTargetSecret},
};
use redis::AsyncCommands;
use redis_args::ToRedisArgs;
use snafu::{OptionExt, ResultExt};

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
}

/// Stores the [`RecordingStatus`] of this room.
#[derive(ToRedisArgs)]
#[to_redis_args(fmt = "opentalk-signaling:room={room}:recording:streams")]
struct RecordingStreamsKey {
    room: SignalingRoomId,
}

pub(crate) async fn set_stream(
    redis_conn: &mut RedisConnection,
    room: SignalingRoomId,
    target: StreamingTargetId,
    stream_target: StreamTargetSecret,
) -> Result<(), SignalingModuleError> {
    redis_conn
        .hset(RecordingStreamsKey { room }, target, stream_target)
        .await
        .context(RedisSnafu {
            message: "Failed to set target stream",
        })
}

pub(crate) async fn get_streams(
    redis_conn: &mut RedisConnection,
    room: SignalingRoomId,
) -> Result<BTreeMap<StreamingTargetId, StreamTargetSecret>, SignalingModuleError> {
    redis_conn
        .hgetall(RecordingStreamsKey { room })
        .await
        .context(RedisSnafu {
            message: "Failed to get all streams",
        })
}

pub(crate) async fn get_stream(
    redis_conn: &mut RedisConnection,
    room: SignalingRoomId,
    target: StreamingTargetId,
) -> Result<StreamTargetSecret, SignalingModuleError> {
    redis_conn
        .hget(RecordingStreamsKey { room }, target)
        .await
        .context(RedisSnafu {
            message: "Failed to get target stream",
        })
}

pub(crate) async fn stream_exists(
    redis_conn: &mut RedisConnection,
    room: SignalingRoomId,
    target: StreamingTargetId,
) -> Result<bool, SignalingModuleError> {
    redis_conn
        .hexists(RecordingStreamsKey { room }, target)
        .await
        .context(RedisSnafu {
            message: "Failed to check for presence of stream",
        })
}

pub(crate) async fn streams_contains_status(
    redis_conn: &mut RedisConnection,
    room: SignalingRoomId,
    status: Vec<StreamStatus>,
) -> Result<bool, SignalingModuleError> {
    let res: BTreeMap<StreamingTargetId, StreamTargetSecret> = redis_conn
        .hgetall(RecordingStreamsKey { room })
        .await
        .context(RedisSnafu {
            message: "Failed to check for status in streams",
        })?;

    let res = res.iter().any(|(_, s)| status.iter().contains(&s.status));

    Ok(res)
}

pub(crate) async fn update_streams(
    redis_conn: &mut RedisConnection,
    room: SignalingRoomId,
    targets: &BTreeSet<StreamingTargetId>,
    status: StreamStatus,
) -> Result<(), SignalingModuleError> {
    let mut streams = get_streams(redis_conn, room).await?;
    let streams = targets
        .iter()
        .map(|id| {
            let mut stream_target = streams
                .remove(id)
                .with_whatever_context::<_, _, SignalingModuleError>(|| {
                    format!("Requested id: '{id}' not found")
                })?;
            stream_target.status = status.clone();
            Ok((*id, stream_target))
        })
        .collect::<Result<BTreeMap<_, _>, SignalingModuleError>>()?;

    redis_conn.set_streams(room, &streams).await
}

pub(crate) async fn delete_all_streams(
    redis_conn: &mut RedisConnection,
    room: SignalingRoomId,
) -> Result<(), SignalingModuleError> {
    redis_conn
        .del(RecordingStreamsKey { room })
        .await
        .context(RedisSnafu {
            message: "Failed to delete recording state",
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
    async fn streams() {
        test_common::streams(&mut storage().await).await;
    }
}
