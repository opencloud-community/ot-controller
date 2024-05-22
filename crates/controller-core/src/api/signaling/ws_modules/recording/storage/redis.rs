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
impl RecordingStorage for RedisConnection {}

/// Stores the [`RecordingStatus`] of this room.
#[derive(ToRedisArgs)]
#[to_redis_args(fmt = "opentalk-signaling:room={room_id}:recording:streams")]
struct RecordingStreamsKey {
    room_id: SignalingRoomId,
}

pub(crate) async fn is_streaming_initialized(
    redis_conn: &mut RedisConnection,
    room_id: SignalingRoomId,
) -> Result<bool, SignalingModuleError> {
    redis_conn
        .exists(RecordingStreamsKey { room_id })
        .await
        .context(RedisSnafu {
            message: "Failed to initialize streaming",
        })
}

pub(crate) async fn set_streams(
    redis_conn: &mut RedisConnection,
    room_id: SignalingRoomId,
    target_stream_ids: &BTreeMap<StreamingTargetId, StreamTargetSecret>,
) -> Result<(), SignalingModuleError> {
    let target_streams: Vec<_> = target_stream_ids.iter().collect();
    redis_conn
        .hset_multiple(RecordingStreamsKey { room_id }, target_streams.as_slice())
        .await
        .context(RedisSnafu {
            message: "Failed to set target stream ids",
        })
}

pub(crate) async fn set_stream(
    redis_conn: &mut RedisConnection,
    room_id: SignalingRoomId,
    target_id: StreamingTargetId,
    stream_target: StreamTargetSecret,
) -> Result<(), SignalingModuleError> {
    redis_conn
        .hset(RecordingStreamsKey { room_id }, target_id, stream_target)
        .await
        .context(RedisSnafu {
            message: "Failed to set target stream",
        })
}

pub(crate) async fn get_streams(
    redis_conn: &mut RedisConnection,
    room_id: SignalingRoomId,
) -> Result<BTreeMap<StreamingTargetId, StreamTargetSecret>, SignalingModuleError> {
    redis_conn
        .hgetall(RecordingStreamsKey { room_id })
        .await
        .context(RedisSnafu {
            message: "Failed to get all streams",
        })
}

pub(crate) async fn get_stream(
    redis_conn: &mut RedisConnection,
    room_id: SignalingRoomId,
    target_id: StreamingTargetId,
) -> Result<StreamTargetSecret, SignalingModuleError> {
    redis_conn
        .hget(RecordingStreamsKey { room_id }, target_id)
        .await
        .context(RedisSnafu {
            message: "Failed to get target stream",
        })
}

pub(crate) async fn stream_exists(
    redis_conn: &mut RedisConnection,
    room_id: SignalingRoomId,
    target_id: StreamingTargetId,
) -> Result<bool, SignalingModuleError> {
    redis_conn
        .hexists(RecordingStreamsKey { room_id }, target_id)
        .await
        .context(RedisSnafu {
            message: "Failed to check for presence of stream",
        })
}

pub(crate) async fn streams_contains_status(
    redis_conn: &mut RedisConnection,
    room_id: SignalingRoomId,
    status: Vec<StreamStatus>,
) -> Result<bool, SignalingModuleError> {
    let res: BTreeMap<StreamingTargetId, StreamTargetSecret> = redis_conn
        .hgetall(RecordingStreamsKey { room_id })
        .await
        .context(RedisSnafu {
            message: "Failed to check for status in streams",
        })?;

    let res = res.iter().any(|(_, s)| status.iter().contains(&s.status));

    Ok(res)
}

pub(crate) async fn update_streams(
    redis_conn: &mut RedisConnection,
    room_id: SignalingRoomId,
    target_ids: &BTreeSet<StreamingTargetId>,
    status: StreamStatus,
) -> Result<(), SignalingModuleError> {
    let mut streams = get_streams(redis_conn, room_id).await?;
    let streams = target_ids
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

    set_streams(redis_conn, room_id, &streams).await
}

pub(crate) async fn delete_all_streams(
    redis_conn: &mut RedisConnection,
    room_id: SignalingRoomId,
) -> Result<(), SignalingModuleError> {
    redis_conn
        .del(RecordingStreamsKey { room_id })
        .await
        .context(RedisSnafu {
            message: "Failed to delete recording state",
        })
}
