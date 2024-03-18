// SPDX-FileCopyrightText: OpenTalk GmbH <mail@opentalk.eu>
//
// SPDX-License-Identifier: EUPL-1.2

use opentalk_signaling_core::{RedisConnection, RedisSnafu, SignalingModuleError, SignalingRoomId};
use opentalk_types::{core::ParticipantId, signaling::timer::ready_status::ReadyStatus};
use redis::AsyncCommands;
use redis_args::ToRedisArgs;
use snafu::ResultExt;

/// A key to track the participants ready status
#[derive(ToRedisArgs)]
#[to_redis_args(
    fmt = "opentalk-signaling:room={room_id}:participant::{participant_id}::timer-ready-status"
)]
struct ReadyStatusKey {
    room_id: SignalingRoomId,
    participant_id: ParticipantId,
}

/// Set the ready status of a participant
#[tracing::instrument(name = "meeting_timer_ready_set", skip(redis_conn))]
pub(crate) async fn set(
    redis_conn: &mut RedisConnection,
    room_id: SignalingRoomId,
    participant_id: ParticipantId,
    ready_status: bool,
) -> Result<(), SignalingModuleError> {
    redis_conn
        .set(
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

/// Get the ready status of a participant
#[tracing::instrument(name = "meeting_timer_ready_get", skip(redis_conn))]
pub(crate) async fn get(
    redis_conn: &mut RedisConnection,
    room_id: SignalingRoomId,
    participant_id: ParticipantId,
) -> Result<Option<ReadyStatus>, SignalingModuleError> {
    redis_conn
        .get(ReadyStatusKey {
            room_id,
            participant_id,
        })
        .await
        .context(RedisSnafu {
            message: "Failed to get ready state",
        })
}

/// Delete the ready status of a participant
#[tracing::instrument(name = "meeting_timer_ready_delete", skip(redis_conn))]
pub(crate) async fn delete(
    redis_conn: &mut RedisConnection,
    room_id: SignalingRoomId,
    participant_id: ParticipantId,
) -> Result<(), SignalingModuleError> {
    redis_conn
        .del(ReadyStatusKey {
            room_id,
            participant_id,
        })
        .await
        .context(RedisSnafu {
            message: "Failed to delete ready state",
        })
}
