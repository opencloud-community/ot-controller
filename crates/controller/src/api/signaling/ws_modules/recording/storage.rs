// SPDX-FileCopyrightText: OpenTalk GmbH <mail@opentalk.eu>
//
// SPDX-License-Identifier: EUPL-1.2

use anyhow::{Context, Result};
use redis::AsyncCommands;
use redis_args::ToRedisArgs;
use signaling_core::{RedisConnection, SignalingRoomId};
use types::signaling::recording::RecordingStatus;

use super::RecordingId;

/// Stores the [`RecordingStatus`] of this room.
#[derive(ToRedisArgs)]
#[to_redis_args(fmt = "opentalk-signaling:room={room_id}:recording:init")]
struct RecordingStateKey {
    room_id: SignalingRoomId,
}

pub(super) async fn try_init(
    redis_conn: &mut RedisConnection,
    room_id: SignalingRoomId,
) -> Result<bool> {
    redis_conn
        .set_nx(RecordingStateKey { room_id }, RecordingStatus::Initializing)
        .await
        .context("Failed to initialize recording state")
}

pub(super) async fn set_recording(
    redis_conn: &mut RedisConnection,
    room_id: SignalingRoomId,
    id: RecordingId,
) -> Result<()> {
    redis_conn
        .set(
            RecordingStateKey { room_id },
            RecordingStatus::Recording(id),
        )
        .await
        .context("Failed to set recording state to 'recording'")
}

pub(super) async fn get_state(
    redis_conn: &mut RedisConnection,
    room_id: SignalingRoomId,
) -> Result<Option<RecordingStatus>> {
    redis_conn
        .get(RecordingStateKey { room_id })
        .await
        .context("Failed to get recording state")
}

pub(super) async fn del_state(
    redis_conn: &mut RedisConnection,
    room_id: SignalingRoomId,
) -> Result<()> {
    redis_conn
        .del(RecordingStateKey { room_id })
        .await
        .context("Failed to delete recording state")
}
