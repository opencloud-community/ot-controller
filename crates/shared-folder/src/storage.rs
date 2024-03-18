// SPDX-FileCopyrightText: OpenTalk GmbH <mail@opentalk.eu>
//
// SPDX-License-Identifier: EUPL-1.2

use opentalk_signaling_core::{RedisConnection, RedisSnafu, SignalingModuleError, SignalingRoomId};
use opentalk_types::common::shared_folder::SharedFolder;
use redis::AsyncCommands;
use redis_args::ToRedisArgs;
use snafu::ResultExt;

#[derive(ToRedisArgs)]
#[to_redis_args(fmt = "opentalk-signaling:room={room}:shared-folder:initialized")]
struct RoomSharedFolderInitialized {
    room: SignalingRoomId,
}

#[tracing::instrument(level = "debug", skip(redis_conn))]
pub async fn set_shared_folder_initialized(
    redis_conn: &mut RedisConnection,
    room: SignalingRoomId,
) -> Result<bool, SignalingModuleError> {
    redis_conn
        .set(RoomSharedFolderInitialized { room }, true)
        .await
        .context(RedisSnafu {
            message: "Failed to SET shared folder initialized flag",
        })
}

#[tracing::instrument(level = "debug", skip(redis_conn))]
pub async fn is_shared_folder_initialized(
    redis_conn: &mut RedisConnection,
    room: SignalingRoomId,
) -> Result<bool, SignalingModuleError> {
    redis_conn
        .get(RoomSharedFolderInitialized { room })
        .await
        .context(RedisSnafu {
            message: "Failed to GET shared folder initialized flag",
        })
}

#[tracing::instrument(level = "debug", skip(redis_conn))]
pub async fn delete_shared_folder_initialized(
    redis_conn: &mut RedisConnection,
    room: SignalingRoomId,
) -> Result<bool, SignalingModuleError> {
    redis_conn
        .del(RoomSharedFolderInitialized { room })
        .await
        .context(RedisSnafu {
            message: "Failed to DEL shared folder initialized flag",
        })
}

/// Key to the shared folder information
#[derive(ToRedisArgs)]
#[to_redis_args(fmt = "opentalk-signaling:room={room}:shared-folder")]
struct RoomSharedFolder {
    room: SignalingRoomId,
}

#[tracing::instrument(level = "debug", skip(redis_conn))]
pub async fn get_shared_folder(
    redis_conn: &mut RedisConnection,
    room: SignalingRoomId,
) -> Result<Option<SharedFolder>, SignalingModuleError> {
    redis_conn
        .get(RoomSharedFolder { room })
        .await
        .context(RedisSnafu {
            message: "Failed to GET shared folder",
        })
}

#[tracing::instrument(level = "debug", skip(redis_conn))]
pub async fn set_shared_folder(
    redis_conn: &mut RedisConnection,
    room: SignalingRoomId,
    value: SharedFolder,
) -> Result<(), SignalingModuleError> {
    redis_conn
        .set(RoomSharedFolder { room }, value)
        .await
        .context(RedisSnafu {
            message: "Failed to SET shared folder",
        })
}

#[tracing::instrument(level = "debug", skip(redis_conn))]
pub async fn delete_shared_folder(
    redis_conn: &mut RedisConnection,
    room: SignalingRoomId,
) -> Result<(), SignalingModuleError> {
    redis_conn
        .del(RoomSharedFolder { room })
        .await
        .context(RedisSnafu {
            message: "Failed to DEL shared folder",
        })
}
