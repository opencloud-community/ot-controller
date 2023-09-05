// SPDX-FileCopyrightText: OpenTalk GmbH <mail@opentalk.eu>
//
// SPDX-License-Identifier: EUPL-1.2

use super::PollsState;
use crate::{ChoiceId, PollId};
use anyhow::{bail, Context, Result};
use redis::AsyncCommands;
use redis_args::ToRedisArgs;
use signaling_core::{RedisConnection, SignalingRoomId};
use std::collections::HashMap;
use types::signaling::polls::Item;

/// Key to the current poll config
#[derive(ToRedisArgs)]
#[to_redis_args(fmt = "opentalk-signaling:room={room}:polls:state")]
struct PollsStateKey {
    room: SignalingRoomId,
}

#[tracing::instrument(level = "debug", skip(redis_conn))]
pub(super) async fn get_state(
    redis_conn: &mut RedisConnection,
    room: SignalingRoomId,
) -> Result<Option<PollsState>> {
    redis_conn
        .get(PollsStateKey { room })
        .await
        .context("failed to get current polls state")
}

/// Set the current polls state if one doesn't already exist returns true if set was successful
#[tracing::instrument(level = "debug", skip(redis_conn))]
pub(super) async fn set_state(
    redis_conn: &mut RedisConnection,
    room: SignalingRoomId,
    polls_state: &PollsState,
) -> Result<bool> {
    let value: redis::Value = redis::cmd("SET")
        .arg(PollsStateKey { room })
        .arg(polls_state)
        .arg("EX")
        .arg(polls_state.duration.as_secs())
        .arg("NX")
        .query_async(redis_conn)
        .await
        .context("failed to set current polls state")?;

    match value {
        redis::Value::Okay => Ok(true),
        redis::Value::Nil => Ok(false),
        _ => bail!("got invalid value from SET EX NX: {:?}", value),
    }
}

#[tracing::instrument(level = "debug", skip(redis_conn))]
pub(super) async fn del_state(
    redis_conn: &mut RedisConnection,
    room: SignalingRoomId,
) -> Result<()> {
    redis_conn
        .del(PollsStateKey { room })
        .await
        .context("failed to del current polls state")
}

/// Key to the current vote results
#[derive(ToRedisArgs)]
#[to_redis_args(fmt = "opentalk-signaling:room={room}:poll={poll}:results")]
struct PollResults {
    room: SignalingRoomId,
    poll: PollId,
}

pub(super) async fn del_results(
    redis_conn: &mut RedisConnection,
    room: SignalingRoomId,
    poll_id: PollId,
) -> Result<()> {
    redis_conn
        .del(PollResults {
            room,
            poll: poll_id,
        })
        .await
        .context("failed to delete results")
}

pub(super) async fn vote(
    redis_conn: &mut RedisConnection,
    room: SignalingRoomId,
    poll_id: PollId,
    previous_choice_id: Option<ChoiceId>,
    new_choice_id: Option<ChoiceId>,
) -> Result<()> {
    // Revoke any previous vote.
    if let Some(choice_id) = previous_choice_id {
        redis_conn
            .zincr(
                PollResults {
                    room,
                    poll: poll_id,
                },
                u32::from(choice_id),
                -1,
            )
            .await
            .context("failed to cast previous vote")?;
    }

    // Apply any new vote.
    if let Some(choice_id) = new_choice_id {
        redis_conn
            .zincr(
                PollResults {
                    room,
                    poll: poll_id,
                },
                u32::from(choice_id),
                1,
            )
            .await
            .context("failed to cast new vote")?;
    }

    Ok(())
}

async fn results(
    redis_conn: &mut RedisConnection,
    room: SignalingRoomId,
    poll: PollId,
) -> Result<HashMap<ChoiceId, u32>> {
    redis_conn
        .zrange_withscores(PollResults { room, poll }, 0, -1)
        .await
        .context("failed to zrange vote results")
}

pub(super) async fn poll_results(
    redis_conn: &mut RedisConnection,
    room: SignalingRoomId,
    config: &PollsState,
) -> Result<Vec<Item>> {
    let votes = results(redis_conn, room, config.id).await?;

    let votes = (0..config.choices.len())
        .map(|i| {
            let id = ChoiceId::from(i as u32);
            let count = votes.get(&id).copied().unwrap_or_default();
            Item { id, count }
        })
        .collect();

    Ok(votes)
}

/// Key to the list of all polls inside the given room
#[derive(ToRedisArgs)]
#[to_redis_args(fmt = "opentalk-signaling:room={room}:polls:list")]
struct PollList {
    room: SignalingRoomId,
}

/// Add a poll to the list
pub(super) async fn list_add(
    redis_conn: &mut RedisConnection,
    room: SignalingRoomId,
    poll_id: PollId,
) -> Result<()> {
    redis_conn
        .sadd(PollList { room }, poll_id)
        .await
        .context("failed to sadd poll list")
}

/// Get all polls for the room
pub(super) async fn list_members(
    redis_conn: &mut RedisConnection,
    room: SignalingRoomId,
) -> Result<Vec<PollId>> {
    redis_conn
        .smembers(PollList { room })
        .await
        .context("failed to get members from poll list")
}
