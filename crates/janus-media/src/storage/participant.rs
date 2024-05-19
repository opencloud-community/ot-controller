// SPDX-FileCopyrightText: OpenTalk GmbH <mail@opentalk.eu>
//
// SPDX-License-Identifier: EUPL-1.2

use opentalk_signaling_core::{RedisConnection, RedisSnafu, SignalingModuleError, SignalingRoomId};
use opentalk_types::core::ParticipantId;
use redis::AsyncCommands;
use redis_args::ToRedisArgs;
use snafu::ResultExt;

/// Data related to a module inside a participant
#[derive(ToRedisArgs)]
#[to_redis_args(
    fmt = "opentalk-signaling:room={room}:participant={participant}:namespace=media:state"
)]
pub(crate) struct ParticipantMediaStateKey {
    pub(crate) room: SignalingRoomId,
    pub(crate) participant: ParticipantId,
}

#[tracing::instrument(level = "debug", skip(redis_conn))]
pub async fn delete_media_state(
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
