// SPDX-FileCopyrightText: OpenTalk GmbH <mail@opentalk.eu>
//
// SPDX-License-Identifier: EUPL-1.2

use anyhow::{Context, Result};
use redis::AsyncCommands;
use redis_args::ToRedisArgs;
use signaling_core::{RedisConnection, SignalingRoomId};
use types::{core::ParticipantId, signaling::media::ParticipantMediaState};

/// Data related to a module inside a participant
// TODO can this be removed?
#[derive(ToRedisArgs)]
#[to_redis_args(
    fmt = "opentalk-signaling:room={room}:participant={participant}:namespace=media:state"
)]
struct ParticipantMediaStateKey {
    room: SignalingRoomId,
    participant: ParticipantId,
}

#[tracing::instrument(level = "debug", skip(redis_conn))]
pub async fn get_participant_media_state(
    redis_conn: &mut RedisConnection,
    room: SignalingRoomId,
    participant: ParticipantId,
) -> Result<Option<ParticipantMediaState>> {
    let json: Option<Vec<u8>> = redis_conn
        .get(ParticipantMediaStateKey { room, participant })
        .await
        .context("Failed to get media state")?;

    if let Some(json) = json {
        serde_json::from_slice(&json).context("Failed to convert json to media state")
    } else {
        Ok(None)
    }
}

#[tracing::instrument(level = "debug", skip(redis_conn))]
pub async fn set_participant_media_state(
    redis_conn: &mut RedisConnection,
    room: SignalingRoomId,
    participant: ParticipantId,
    state: &ParticipantMediaState,
) -> Result<()> {
    let json = serde_json::to_vec(&state).context("Failed to convert media state to json")?;

    redis_conn
        .set(ParticipantMediaStateKey { room, participant }, json)
        .await
        .context("Failed to get media state")?;

    Ok(())
}

#[tracing::instrument(level = "debug", skip(redis_conn))]
pub async fn del_participant_media_state(
    redis_conn: &mut RedisConnection,
    room: SignalingRoomId,
    participant: ParticipantId,
) -> Result<()> {
    redis_conn
        .del(ParticipantMediaStateKey { room, participant })
        .await
        .context("Failed to delete media state")
}

#[derive(ToRedisArgs)]
#[to_redis_args(fmt = "opentalk-signaling:room={room}:namespace=media:presenters")]
struct Presenters {
    room: SignalingRoomId,
}

#[tracing::instrument(level = "debug", skip(redis_conn))]
pub async fn set_presenter(
    redis_conn: &mut RedisConnection,
    room: SignalingRoomId,
    participant: ParticipantId,
) -> Result<()> {
    redis_conn.sadd(Presenters { room }, participant).await?;

    Ok(())
}

#[tracing::instrument(level = "debug", skip(redis_conn))]
pub async fn is_presenter(
    redis_conn: &mut RedisConnection,
    room: SignalingRoomId,
    participant: ParticipantId,
) -> Result<bool> {
    let value: bool = redis_conn
        .sismember(Presenters { room }, participant)
        .await?;

    Ok(value)
}

#[tracing::instrument(level = "debug", skip(redis_conn))]
pub async fn delete_presenter(
    redis_conn: &mut RedisConnection,
    room: SignalingRoomId,
    participant: ParticipantId,
) -> Result<()> {
    redis_conn.srem(Presenters { room }, participant).await?;

    Ok(())
}

#[tracing::instrument(level = "debug", skip(redis_conn))]
pub async fn delete_presenter_key(
    redis_conn: &mut RedisConnection,
    room: SignalingRoomId,
) -> Result<()> {
    redis_conn.del(Presenters { room }).await?;

    Ok(())
}
