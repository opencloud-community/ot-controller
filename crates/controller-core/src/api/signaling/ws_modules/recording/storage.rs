// SPDX-FileCopyrightText: OpenTalk GmbH <mail@opentalk.eu>
//
// SPDX-License-Identifier: EUPL-1.2

use std::collections::{BTreeMap, BTreeSet};

use anyhow::{Context, Result};
use itertools::Itertools;
use opentalk_signaling_core::{RedisConnection, SignalingRoomId};
use opentalk_types::{
    core::StreamingTargetId,
    signaling::recording::{StreamStatus, StreamTargetSecret},
};
use redis::AsyncCommands;
use redis_args::ToRedisArgs;
use snafu::ResultExt;

/// Stores the [`RecordingStatus`] of this room.
#[derive(ToRedisArgs)]
#[to_redis_args(fmt = "opentalk-signaling:room={room_id}:recording:streams")]
struct RecordingStreamsKey {
    room_id: SignalingRoomId,
}

pub(super) async fn is_streaming_initialized(
    redis_conn: &mut RedisConnection,
    room_id: SignalingRoomId,
) -> Result<bool, SignalingModuleError> {
    redis_conn
        .exists(RecordingStreamsKey { room_id })
        .await
        .context(RedisSnafu {
            message: "Failed to initialize recording state",
        })
}

pub(crate) async fn set_streams(
    redis_conn: &mut RedisConnection,
    room_id: SignalingRoomId,
    id: RecordingId,
) -> Result<(), SignalingModuleError> {
    redis_conn
        .hset(RecordingStreamsKey { room_id }, target_id, stream_target)
        .await
        .context(RedisSnafu {
            message: "Failed to set recording state to 'recording'",
        })
}

pub(crate) async fn get_streams(
    redis_conn: &mut RedisConnection,
    room_id: SignalingRoomId,
) -> Result<BTreeMap<StreamingTargetId, StreamTargetSecret>> {
    redis_conn
        .hgetall(RecordingStreamsKey { room_id })
        .await
        .context(RedisSnafu {
            message: "Failed to get recording state",
        })
}

pub(crate) async fn get_stream(
    redis_conn: &mut RedisConnection,
    room_id: SignalingRoomId,
    target_id: StreamingTargetId,
) -> Result<StreamTargetSecret> {
    redis_conn
        .hget(RecordingStreamsKey { room_id }, target_id)
        .await
        .context("Failed to get target stream")
}

pub(super) async fn stream_exists(
    redis_conn: &mut RedisConnection,
    room_id: SignalingRoomId,
    target_id: StreamingTargetId,
) -> Result<bool> {
    redis_conn
        .hexists(RecordingStreamsKey { room_id }, target_id)
        .await
        .context("Failed to check for presence of stream")
}

pub(super) async fn streams_contains_status(
    redis_conn: &mut RedisConnection,
    room_id: SignalingRoomId,
    status: Vec<StreamStatus>,
) -> Result<bool> {
    let res: BTreeMap<StreamingTargetId, StreamTargetSecret> = redis_conn
        .hgetall(RecordingStreamsKey { room_id })
        .await
        .context("Failed to check for status in streams")?;

    let res = res.iter().any(|(_, s)| status.iter().contains(&s.status));

    Ok(res)
}

pub(super) async fn update_streams(
    redis_conn: &mut RedisConnection,
    room_id: SignalingRoomId,
    target_ids: &BTreeSet<StreamingTargetId>,
    status: StreamStatus,
) -> Result<()> {
    let mut streams = get_streams(redis_conn, room_id).await?;
    let streams = target_ids
        .iter()
        .map(|id| {
            let mut stream_target = streams.remove(id).context("Requested id(s) not found")?;
            stream_target.status = status.clone();
            Ok((*id, stream_target))
        })
        .collect::<Result<BTreeMap<StreamingTargetId, StreamTargetSecret>>>()?;

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
