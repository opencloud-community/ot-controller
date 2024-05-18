// SPDX-FileCopyrightText: OpenTalk GmbH <mail@opentalk.eu>
//
// SPDX-License-Identifier: EUPL-1.2

use opentalk_signaling_core::{RedisConnection, RedisSnafu, SignalingModuleError};
use opentalk_types::core::RoomId;
use redis::AsyncCommands as _;
use redis_args::ToRedisArgs;
use snafu::ResultExt as _;

use super::BreakoutConfig;

/// Typed key to the breakout-room config for the specified room
#[derive(ToRedisArgs)]
#[to_redis_args(fmt = "opentalk-signaling:room={room}:breakout:config")]
struct BreakoutRoomConfig {
    room: RoomId,
}

pub async fn set_config(
    redis_conn: &mut RedisConnection,
    room: RoomId,
    config: &BreakoutConfig,
) -> Result<(), SignalingModuleError> {
    if let Some(duration) = config.duration {
        redis_conn
            .set_ex(BreakoutRoomConfig { room }, config, duration.as_secs())
            .await
            .context(RedisSnafu {
                message: "Failed to set breakout-room config",
            })
    } else {
        redis_conn
            .set(BreakoutRoomConfig { room }, config)
            .await
            .context(RedisSnafu {
                message: "Failed to set breakout-room config",
            })
    }
}

pub async fn get_config(
    redis_conn: &mut RedisConnection,
    room: RoomId,
) -> Result<Option<BreakoutConfig>, SignalingModuleError> {
    redis_conn
        .get(BreakoutRoomConfig { room })
        .await
        .context(RedisSnafu {
            message: "Failed to get breakout-room config",
        })
}

pub async fn del_config(
    redis_conn: &mut RedisConnection,
    room: RoomId,
) -> Result<bool, SignalingModuleError> {
    redis_conn
        .del(BreakoutRoomConfig { room })
        .await
        .context(RedisSnafu {
            message: "Failed to del breakout-room config",
        })
}
