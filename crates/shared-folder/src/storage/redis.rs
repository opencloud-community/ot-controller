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

    #[tracing::instrument(level = "debug", skip(self))]
    async fn is_shared_folder_initialized(
        &mut self,
        room: SignalingRoomId,
    ) -> Result<bool, SignalingModuleError> {
        self.get(RoomSharedFolderInitialized { room })
            .await
            .context(RedisSnafu {
                message: "Failed to GET shared folder initialized flag",
            })
    }

    #[tracing::instrument(level = "debug", skip(self))]
    async fn delete_shared_folder_initialized(
        &mut self,
        room: SignalingRoomId,
    ) -> Result<(), SignalingModuleError> {
        self.del(RoomSharedFolderInitialized { room })
            .await
            .context(RedisSnafu {
                message: "Failed to DEL shared folder initialized flag",
            })
    }

    #[tracing::instrument(level = "debug", skip(self))]
    async fn get_shared_folder(
        &mut self,
        room: SignalingRoomId,
    ) -> Result<Option<SharedFolder>, SignalingModuleError> {
        self.get(RoomSharedFolder { room })
            .await
            .context(RedisSnafu {
                message: "Failed to GET shared folder",
            })
    }

    #[tracing::instrument(level = "debug", skip(self))]
    async fn set_shared_folder(
        &mut self,
        room: SignalingRoomId,
        value: SharedFolder,
    ) -> Result<(), SignalingModuleError> {
        self.set(RoomSharedFolder { room }, value)
            .await
            .context(RedisSnafu {
                message: "Failed to SET shared folder",
            })
    }

    #[tracing::instrument(level = "debug", skip(self))]
    async fn delete_shared_folder(
        &mut self,
        room: SignalingRoomId,
    ) -> Result<(), SignalingModuleError> {
        self.del(RoomSharedFolder { room })
            .await
            .context(RedisSnafu {
                message: "Failed to DEL shared folder",
            })
    }
}

#[derive(ToRedisArgs)]
#[to_redis_args(fmt = "opentalk-signaling:room={room}:shared-folder:initialized")]
struct RoomSharedFolderInitialized {
    room: SignalingRoomId,
}

/// Key to the shared folder information
#[derive(ToRedisArgs)]
#[to_redis_args(fmt = "opentalk-signaling:room={room}:shared-folder")]
struct RoomSharedFolder {
    room: SignalingRoomId,
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

    #[tokio::test]
    #[serial]
    async fn shared_folder() {
        test_common::shared_folder(&mut storage().await).await;
    }
}
