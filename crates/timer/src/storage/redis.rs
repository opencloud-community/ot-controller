// SPDX-FileCopyrightText: OpenTalk GmbH <mail@opentalk.eu>
//
// SPDX-License-Identifier: EUPL-1.2

use opentalk_signaling_core::{RedisConnection, RedisSnafu, SignalingModuleError, SignalingRoomId};
use opentalk_types::{core::ParticipantId, signaling::timer::ready_status::ReadyStatus};
use redis::AsyncCommands;
use redis_args::ToRedisArgs;
use snafu::ResultExt;

use super::Timer;

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
pub(crate) async fn ready_status_set(
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
pub(crate) async fn ready_status_get(
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
pub(crate) async fn ready_status_delete(
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

/// The timer key holds a serialized [`Timer`].
#[derive(ToRedisArgs)]
#[to_redis_args(fmt = "opentalk-signaling:room={room_id}:timer")]
struct TimerKey {
    room_id: SignalingRoomId,
}

/// Attempt to set a new timer
///
/// Returns `true` when the new timer was created
/// Returns `false` when a timer is already active
#[tracing::instrument(name = "meeting_timer_set", skip(redis_conn, timer))]
pub(crate) async fn timer_set_if_not_exists(
    redis_conn: &mut RedisConnection,
    room_id: SignalingRoomId,
    timer: &Timer,
) -> Result<bool, SignalingModuleError> {
    redis_conn
        .set_nx(TimerKey { room_id }, timer)
        .await
        .context(RedisSnafu {
            message: "Failed to set meeting timer",
        })
}

/// Get the current meeting timer
#[tracing::instrument(name = "meeting_timer_get", skip(redis_conn))]
pub(crate) async fn timer_get(
    redis_conn: &mut RedisConnection,
    room_id: SignalingRoomId,
) -> Result<Option<Timer>, SignalingModuleError> {
    redis_conn
        .get(TimerKey { room_id })
        .await
        .context(RedisSnafu {
            message: "Failed to get meeting timer",
        })
}

/// Delete the current timer
///
/// Returns the timer if there was any
#[tracing::instrument(name = "meeting_timer_delete", skip(redis_conn))]
pub(crate) async fn timer_delete(
    redis_conn: &mut RedisConnection,
    room_id: SignalingRoomId,
) -> Result<Option<Timer>, SignalingModuleError> {
    redis::cmd("GETDEL")
        .arg(TimerKey { room_id })
        .query_async(redis_conn)
        .await
        .context(RedisSnafu {
            message: "Failed to delete meeting timer",
        })
}
