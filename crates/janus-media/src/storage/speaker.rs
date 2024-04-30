// SPDX-FileCopyrightText: OpenTalk GmbH <mail@opentalk.eu>
//
// SPDX-License-Identifier: EUPL-1.2

use opentalk_signaling_core::{RedisConnection, RedisSnafu, SignalingModuleError, SignalingRoomId};
use opentalk_types::{
    core::{ParticipantId, Timestamp},
    signaling::media::{ParticipantSpeakingState, SpeakingState},
};
use redis::AsyncCommands;
use redis_args::ToRedisArgs;
use snafu::ResultExt;

/// Data related to a module inside a participant
#[derive(ToRedisArgs)]
#[to_redis_args(
    fmt = "opentalk-signaling:room={room}:participant={participant}:namespace=media:speaker"
)]
pub(crate) struct SpeakerKey {
    pub(crate) room: SignalingRoomId,
    pub(crate) participant: ParticipantId,
}

#[tracing::instrument(level = "debug", skip(redis_conn))]
pub async fn set(
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
pub async fn get(
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
pub async fn delete(
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

pub async fn delete_all_for_room(
    redis_conn: &mut RedisConnection,
    room: SignalingRoomId,
    participants: &[ParticipantId],
) -> Result<(), SignalingModuleError> {
    for &participant in participants {
        delete(redis_conn, room, participant).await?;
    }
    Ok(())
}

pub async fn get_all_for_room(
    redis_conn: &mut RedisConnection,
    room: SignalingRoomId,
    participants: &[ParticipantId],
) -> Result<Vec<ParticipantSpeakingState>, SignalingModuleError> {
    let mut participant_speakers = Vec::new();

    for &participant in participants {
        if let Some(speaker) = get(redis_conn, room, participant).await? {
            participant_speakers.push(ParticipantSpeakingState {
                participant,
                speaker,
            });
        }
    }

    Ok(participant_speakers)
}
