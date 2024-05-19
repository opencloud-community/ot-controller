// SPDX-FileCopyrightText: OpenTalk GmbH <mail@opentalk.eu>
//
// SPDX-License-Identifier: EUPL-1.2

use opentalk_signaling_core::{RedisConnection, RedisSnafu, SignalingModuleError, SignalingRoomId};
use redis::AsyncCommands;
use redis_args::ToRedisArgs;
use snafu::ResultExt;

#[derive(ToRedisArgs)]
#[to_redis_args(fmt = "opentalk-signaling:room={room}:namespace=media:presenters")]
pub(crate) struct Presenters {
    pub(crate) room: SignalingRoomId,
}

#[tracing::instrument(level = "debug", skip(redis_conn))]
pub async fn delete_key(
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
