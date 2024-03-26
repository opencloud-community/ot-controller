// SPDX-FileCopyrightText: OpenTalk GmbH <mail@opentalk.eu>
//
// SPDX-License-Identifier: EUPL-1.2

use opentalk_signaling_core::{
    RedisConnection, RedisSnafu, SerdeJsonSnafu, SignalingModuleError, SignalingRoomId,
};
use opentalk_types::{
    core::{ParticipantId, Timestamp},
    signaling::media::{ParticipantMediaState, ParticipantSpeakingState, SpeakingState},
};
use redis::AsyncCommands;
use redis_args::ToRedisArgs;
use snafu::ResultExt;

/// Data related to a module inside a participant
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
) -> Result<Option<ParticipantMediaState>, SignalingModuleError> {
    let json: Option<Vec<u8>> = redis_conn
        .get(ParticipantMediaStateKey { room, participant })
        .await
        .context(RedisSnafu {
            message: "Failed to get media state",
        })?;

    if let Some(json) = json {
        serde_json::from_slice(&json).context(SerdeJsonSnafu {
            message: "Failed to convert json to media state",
        })
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
) -> Result<(), SignalingModuleError> {
    let json = serde_json::to_vec(&state).context(SerdeJsonSnafu {
        message: "Failed to convert media state to json",
    })?;

    redis_conn
        .set(ParticipantMediaStateKey { room, participant }, json)
        .await
        .context(RedisSnafu {
            message: "Failed to get media state",
        })?;

    Ok(())
}

#[tracing::instrument(level = "debug", skip(redis_conn))]
pub async fn del_participant_media_state(
    redis_conn: &mut RedisConnection,
    room: SignalingRoomId,
    participant: ParticipantId,
) -> Result<(), SignalingModuleError> {
    redis_conn
        .del(ParticipantMediaStateKey { room, participant })
        .await
        .context(RedisSnafu {
            message: "Failed to delete media state",
        })
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
) -> Result<(), SignalingModuleError> {
    redis_conn
        .sadd(Presenters { room }, participant)
        .await
        .context(RedisSnafu {
            message: "Failed to set presenter",
        })?;

    Ok(())
}

#[tracing::instrument(level = "debug", skip(redis_conn))]
pub async fn is_presenter(
    redis_conn: &mut RedisConnection,
    room: SignalingRoomId,
    participant: ParticipantId,
) -> Result<bool, SignalingModuleError> {
    let value: bool = redis_conn
        .sismember(Presenters { room }, participant)
        .await
        .context(RedisSnafu {
            message: "Failed to check if participant is presenter",
        })?;

    Ok(value)
}

#[tracing::instrument(level = "debug", skip(redis_conn))]
pub async fn delete_presenter(
    redis_conn: &mut RedisConnection,
    room: SignalingRoomId,
    participant: ParticipantId,
) -> Result<(), SignalingModuleError> {
    redis_conn
        .srem(Presenters { room }, participant)
        .await
        .context(RedisSnafu {
            message: "Failed to delete presenter",
        })?;

    Ok(())
}

#[tracing::instrument(level = "debug", skip(redis_conn))]
pub async fn delete_presenter_key(
    redis_conn: &mut RedisConnection,
    room: SignalingRoomId,
) -> Result<(), SignalingModuleError> {
    redis_conn
        .del(Presenters { room })
        .await
        .context(RedisSnafu {
            message: "Failed to delete presenter key",
        })?;

    Ok(())
}

/// Data related to a module inside a participant
#[derive(ToRedisArgs)]
#[to_redis_args(
    fmt = "opentalk-signaling:room={room}:participant={participant}:namespace=media:speaker"
)]
struct SpeakerKey {
    room: SignalingRoomId,
    participant: ParticipantId,
}

#[tracing::instrument(level = "debug", skip(redis_conn))]
pub async fn set_speaker(
    redis_conn: &mut RedisConnection,
    room: SignalingRoomId,
    participant: ParticipantId,
    is_speaking: bool,
    updated_at: Timestamp,
) -> Result<(), SignalingModuleError> {
    redis_conn
        .set(
            SpeakerKey { room, participant },
            Some(SpeakingState {
                is_speaking,
                updated_at,
            }),
        )
        .await
        .context(RedisSnafu {
            message: "Failed to set speaker state",
        })
}

#[tracing::instrument(level = "debug", skip(redis_conn))]
pub async fn get_speaker(
    redis_conn: &mut RedisConnection,
    room: SignalingRoomId,
    participant: ParticipantId,
) -> Result<Option<SpeakingState>, SignalingModuleError> {
    redis_conn
        .get(SpeakerKey { room, participant })
        .await
        .context(RedisSnafu {
            message: "Failed to get speaker state",
        })
}

#[tracing::instrument(level = "debug", skip(redis_conn))]
pub async fn delete_speaker(
    redis_conn: &mut RedisConnection,
    room: SignalingRoomId,
    participant: ParticipantId,
) -> Result<(), SignalingModuleError> {
    redis_conn
        .del(SpeakerKey { room, participant })
        .await
        .context(RedisSnafu {
            message: "Failed to delete speaker state",
        })
}

pub async fn delete_room_speakers(
    redis_conn: &mut RedisConnection,
    room: SignalingRoomId,
    participants: &[ParticipantId],
) -> Result<(), SignalingModuleError> {
    for &participant in participants {
        delete_speaker(redis_conn, room, participant).await?;
    }
    Ok(())
}

pub async fn get_room_speakers(
    redis_conn: &mut RedisConnection,
    room: SignalingRoomId,
    participants: &[ParticipantId],
) -> Result<Vec<ParticipantSpeakingState>, SignalingModuleError> {
    let mut participant_speakers = Vec::new();

    for &participant in participants {
        if let Some(speaker) = get_speaker(redis_conn, room, participant).await? {
            participant_speakers.push(ParticipantSpeakingState {
                participant,
                speaker,
            });
        }
    }

    Ok(participant_speakers)
}
