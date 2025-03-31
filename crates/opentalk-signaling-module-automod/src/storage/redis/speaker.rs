// SPDX-FileCopyrightText: OpenTalk GmbH <mail@opentalk.eu>
//
// SPDX-License-Identifier: EUPL-1.2

//! Single value entry which holds the participant id of the current speaker.
//!
//! If not set, then there is currently no active speaker.
use async_trait::async_trait;
use opentalk_signaling_core::{RedisConnection, RedisSnafu, SignalingModuleError, SignalingRoomId};
use opentalk_types_signaling::ParticipantId;
use redis::AsyncCommands;
use redis_args::ToRedisArgs;
use snafu::ResultExt;

use crate::storage::automod_storage::AutomodSpeakerStorage;

#[async_trait(?Send)]
impl AutomodSpeakerStorage for RedisConnection {
    #[tracing::instrument(name = "get_speaker", level = "debug", skip(self))]
    async fn speaker_get(
        &mut self,
        room: SignalingRoomId,
    ) -> Result<Option<ParticipantId>, SignalingModuleError> {
        self.get(RoomAutomodSpeaker { room })
            .await
            .context(RedisSnafu {
                message: "Failed to set active speaker",
            })
    }

    #[tracing::instrument(name = "set_speaker", level = "debug", skip(self))]
    async fn speaker_set(
        &mut self,
        room: SignalingRoomId,
        participant: ParticipantId,
    ) -> Result<Option<ParticipantId>, SignalingModuleError> {
        redis::cmd("SET")
            .arg(RoomAutomodSpeaker { room })
            .arg(participant)
            .arg("GET")
            .query_async(self)
            .await
            .context(RedisSnafu {
                message: "Failed to set active speaker",
            })
    }

    #[tracing::instrument(name = "del_speaker", level = "debug", skip(self))]
    async fn speaker_delete(
        &mut self,
        room: SignalingRoomId,
    ) -> Result<Option<ParticipantId>, SignalingModuleError> {
        redis::cmd("GETDEL")
            .arg(RoomAutomodSpeaker { room })
            .query_async(self)
            .await
            .context(RedisSnafu {
                message: "Failed to del active speaker",
            })
    }
}

/// Typed key to the automod's active speaker
#[derive(ToRedisArgs)]
#[to_redis_args(fmt = "opentalk-signaling:room={room}:automod:speaker")]
pub struct RoomAutomodSpeaker {
    room: SignalingRoomId,
}
