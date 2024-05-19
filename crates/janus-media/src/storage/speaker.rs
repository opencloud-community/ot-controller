// SPDX-FileCopyrightText: OpenTalk GmbH <mail@opentalk.eu>
//
// SPDX-License-Identifier: EUPL-1.2

use opentalk_signaling_core::{RedisConnection, SignalingModuleError, SignalingRoomId};
use opentalk_types::{core::ParticipantId, signaling::media::ParticipantSpeakingState};
use redis_args::ToRedisArgs;

use super::MediaStorage as _;

/// Data related to a module inside a participant
#[derive(ToRedisArgs)]
#[to_redis_args(
    fmt = "opentalk-signaling:room={room}:participant={participant}:namespace=media:speaker"
)]
pub(crate) struct SpeakerKey {
    pub(crate) room: SignalingRoomId,
    pub(crate) participant: ParticipantId,
}

pub async fn get_all_for_room(
    redis_conn: &mut RedisConnection,
    room: SignalingRoomId,
    participants: &[ParticipantId],
) -> Result<Vec<ParticipantSpeakingState>, SignalingModuleError> {
    let mut participant_speakers = Vec::new();

    for &participant in participants {
        if let Some(speaker) = redis_conn.get_speaking_state(room, participant).await? {
            participant_speakers.push(ParticipantSpeakingState {
                participant,
                speaker,
            });
        }
    }

    Ok(participant_speakers)
}
