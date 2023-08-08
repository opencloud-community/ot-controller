// SPDX-FileCopyrightText: OpenTalk GmbH <mail@opentalk.eu>
//
// SPDX-License-Identifier: EUPL-1.2

use crate::{Kind, TimerId};

use anyhow::{Context, Result};
use redis::AsyncCommands;
use redis_args::{FromRedisValue, ToRedisArgs};
use serde::{Deserialize, Serialize};
use signaling_core::{RedisConnection, SignalingRoomId};
use types::core::{ParticipantId, Timestamp};

/// The timer key holds a serialized [`Timer`].
#[derive(ToRedisArgs)]
#[to_redis_args(fmt = "opentalk-signaling:room={room_id}:timer")]
struct TimerKey {
    room_id: SignalingRoomId,
}

/// A timer
///
/// Stores information about a running timer
#[derive(Debug, Serialize, Deserialize, ToRedisArgs, FromRedisValue)]
#[to_redis_args(serde)]
#[from_redis_value(serde)]
pub(crate) struct Timer {
    /// The timers id
    ///
    /// Used to match expire events to their respective timer
    pub(crate) id: TimerId,
    /// The creator of the timer
    pub(crate) created_by: ParticipantId,
    /// The start of the timer
    ///
    /// Allows us to calculate the passed duration for joining participants
    pub(crate) started_at: Timestamp,
    /// The Timer kind
    pub(crate) kind: Kind,
    /// An optional string tag to flag this timer with a custom style
    pub(crate) style: Option<String>,
    /// The optional title
    pub(crate) title: Option<String>,
    /// Flag to allow/disallow participants to mark themselves as ready
    pub(crate) ready_check_enabled: bool,
}

/// Attempt to set a new timer
///
/// Returns `true` when the new timer was created
/// Returns `false` when a timer is already active
#[tracing::instrument(name = "meeting_timer_set", skip(redis_conn, timer))]
pub(crate) async fn set_if_not_exists(
    redis_conn: &mut RedisConnection,
    room_id: SignalingRoomId,
    timer: &Timer,
) -> Result<bool> {
    redis_conn
        .set_nx(TimerKey { room_id }, timer)
        .await
        .context("Failed to set meeting timer")
}

/// Get the current meeting timer
#[tracing::instrument(name = "meeting_timer_get", skip(redis_conn))]
pub(crate) async fn get(
    redis_conn: &mut RedisConnection,
    room_id: SignalingRoomId,
) -> Result<Option<Timer>> {
    redis_conn
        .get(TimerKey { room_id })
        .await
        .context("Failed to get meeting timer")
}

/// Delete the current timer
///
/// Returns the timer if there was any
#[tracing::instrument(name = "meeting_timer_delete", skip(redis_conn))]
pub(crate) async fn delete(
    redis_conn: &mut RedisConnection,
    room_id: SignalingRoomId,
) -> Result<Option<Timer>> {
    redis::cmd("GETDEL")
        .arg(TimerKey { room_id })
        .query_async(redis_conn)
        .await
        .context("Failed to delete meeting timer")
}
