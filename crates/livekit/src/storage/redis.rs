// SPDX-FileCopyrightText: OpenTalk GmbH <mail@opentalk.eu>
//
// SPDX-License-Identifier: EUPL-1.2

use std::collections::BTreeSet;

use async_trait::async_trait;
use opentalk_signaling_core::{RedisConnection, RedisSnafu, SignalingModuleError};
use opentalk_types_common::rooms::RoomId;
use opentalk_types_signaling::ParticipantId;
use opentalk_types_signaling_livekit::MicrophoneRestrictionState;
use redis::AsyncCommands as _;
use redis_args::ToRedisArgs;
use snafu::ResultExt;

use super::LivekitStorage;

#[async_trait]
impl LivekitStorage for RedisConnection {
    #[tracing::instrument(level = "debug", skip(self))]
    async fn set_microphone_restriction_allow_list(
        &mut self,
        room: RoomId,
        participant_id: &[ParticipantId],
    ) -> Result<(), SignalingModuleError> {
        if participant_id.is_empty() {
            return self.clear_microphone_restriction(room).await;
        }

        redis::pipe()
            .del(AllowedUnmuteList { room })
            .sadd(AllowedUnmuteList { room }, participant_id)
            .query_async(self)
            .await
            .context(RedisSnafu {
                message: "Failed to SADD allow_unmute_list",
            })
    }

    #[tracing::instrument(level = "debug", skip(self))]
    async fn clear_microphone_restriction(
        &mut self,
        room: RoomId,
    ) -> Result<(), SignalingModuleError> {
        self.del(AllowedUnmuteList { room })
            .await
            .context(RedisSnafu {
                message: "Failed to DEL force_mute state",
            })
    }

    #[tracing::instrument(level = "debug", skip(self))]
    async fn get_microphone_restriction_state(
        &mut self,
        room: RoomId,
    ) -> Result<MicrophoneRestrictionState, SignalingModuleError> {
        let unrestricted_participants: BTreeSet<ParticipantId> = self
            .smembers(AllowedUnmuteList { room })
            .await
            .context(RedisSnafu {
                message: "Failed to SMEMBERS allow_unmute_list",
            })?;

        if unrestricted_participants.is_empty() {
            Ok(MicrophoneRestrictionState::Disabled)
        } else {
            Ok(MicrophoneRestrictionState::Enabled {
                unrestricted_participants,
            })
        }
    }
}

/// The set participants that are allowed to unmute themselves even if the forced mute state is enabled
#[derive(ToRedisArgs)]
#[to_redis_args(fmt = "opentalk-signaling:room={room}:allowed_unmute")]
struct AllowedUnmuteList {
    room: RoomId,
}
