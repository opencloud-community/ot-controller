// SPDX-FileCopyrightText: OpenTalk GmbH <mail@opentalk.eu>
//
// SPDX-License-Identifier: EUPL-1.2

use async_trait::async_trait;
use opentalk_signaling_core::{RedisConnection, RedisSnafu, SignalingModuleError, SignalingRoomId};
use opentalk_types::common::shared_folder::SharedFolder;
use redis::AsyncCommands;
use redis_args::ToRedisArgs;
use snafu::ResultExt;

use super::SharedFolderStorage;

#[async_trait(?Send)]
impl SharedFolderStorage for RedisConnection {
    #[tracing::instrument(level = "debug", skip(self))]
    async fn set_shared_folder_initialized(
        &mut self,
        room: SignalingRoomId,
    ) -> Result<(), SignalingModuleError> {
        self.set(RoomSharedFolderInitialized { room }, true)
            .await
            .context(RedisSnafu {
                message: "Failed to SET shared folder initialized flag",
            })
    }
}

#[derive(ToRedisArgs)]
#[to_redis_args(fmt = "opentalk-signaling:room={room}:shared-folder:initialized")]
struct RoomSharedFolderInitialized {
    room: SignalingRoomId,
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

#[cfg(test)]
mod test {
    use redis::aio::ConnectionManager;
    use serial_test::serial;

    use super::{super::test_common, *};

    async fn storage() -> RedisConnection {
        let redis_url =
            std::env::var("REDIS_ADDR").unwrap_or_else(|_| "redis://0.0.0.0:6379/".to_owned());
        let redis = redis::Client::open(redis_url).expect("Invalid redis url");

        let mut mgr = ConnectionManager::new(redis).await.unwrap();

        redis::cmd("FLUSHALL")
            .query_async::<_, ()>(&mut mgr)
            .await
            .unwrap();

        RedisConnection::new(mgr)
    }

    #[tokio::test]
    #[serial]
    async fn initialized() {
        test_common::initialized(&mut storage().await).await;
    }
}
