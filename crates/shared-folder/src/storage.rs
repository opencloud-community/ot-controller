// SPDX-FileCopyrightText: OpenTalk GmbH <mail@opentalk.eu>
//
// SPDX-License-Identifier: EUPL-1.2

use anyhow::{Context, Result};
use redis::AsyncCommands;
use redis_args::ToRedisArgs;
use signaling_core::{RedisConnection, SignalingRoomId};
use types::common::shared_folder::SharedFolder;

#[derive(ToRedisArgs)]
#[to_redis_args(fmt = "opentalk-signaling:room={room}:shared-folder:initialized")]
struct RoomSharedFolderInitialized {
    room: SignalingRoomId,
}

#[tracing::instrument(level = "debug", skip(redis_conn))]
pub async fn set_shared_folder_initialized(
    redis_conn: &mut RedisConnection,
    room: SignalingRoomId,
) -> Result<bool> {
    redis_conn
        .set(RoomSharedFolderInitialized { room }, true)
        .await
        .context("Failed to SET shared folder initialized flag")
}

#[tracing::instrument(level = "debug", skip(redis_conn))]
pub async fn is_shared_folder_initialized(
    redis_conn: &mut RedisConnection,
    room: SignalingRoomId,
) -> Result<bool> {
    redis_conn
        .get(RoomSharedFolderInitialized { room })
        .await
        .context("Failed to GET shared folder initialized flag")
}

#[tracing::instrument(level = "debug", skip(redis_conn))]
pub async fn delete_shared_folder_initialized(
    redis_conn: &mut RedisConnection,
    room: SignalingRoomId,
) -> Result<bool> {
    redis_conn
        .del(RoomSharedFolderInitialized { room })
        .await
        .context("Failed to DEL shared folder initialized flag")
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
) -> Result<Option<SharedFolder>> {
    redis_conn
        .get(RoomSharedFolder { room })
        .await
        .context("Failed to GET shared folder")
}

#[tracing::instrument(level = "debug", skip(redis_conn))]
pub async fn set_shared_folder(
    redis_conn: &mut RedisConnection,
    room: SignalingRoomId,
    value: SharedFolder,
) -> Result<()> {
    redis_conn
        .set(RoomSharedFolder { room }, value)
        .await
        .context("Failed to SET shared folder")
}

#[tracing::instrument(level = "debug", skip(redis_conn))]
pub async fn delete_shared_folder(
    redis_conn: &mut RedisConnection,
    room: SignalingRoomId,
) -> Result<()> {
    redis_conn
        .del(RoomSharedFolder { room })
        .await
        .context("Failed to DEL shared folder")
}
