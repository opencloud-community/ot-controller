// SPDX-FileCopyrightText: OpenTalk GmbH <mail@opentalk.eu>
//
// SPDX-License-Identifier: EUPL-1.2

//! The configuration of the automod. If it exists inside of redis for a room, the room is
//! considered to being auto-moderated.

use async_trait::async_trait;
use opentalk_signaling_core::{RedisConnection, RedisSnafu, SignalingModuleError, SignalingRoomId};
use redis::AsyncCommands;
use redis_args::ToRedisArgs;
use snafu::ResultExt;

use crate::storage::{automod_storage::AutomodConfigStorage, StorageConfig};

#[async_trait(?Send)]
impl AutomodConfigStorage for RedisConnection {
    /// Set the current config.
    #[tracing::instrument(name = "set_config", level = "debug", skip(self, config))]
    async fn config_set(
        &mut self,
        room: SignalingRoomId,
        config: StorageConfig,
    ) -> Result<(), SignalingModuleError> {
        self.set(RoomAutomodConfig { room }, &config)
            .await
            .context(RedisSnafu {
                message: "Failed to set config",
            })
    }

    /// Get the current config, if any is set.
    ///
    /// If it returns `Some`, one must assume the automod is active.
    #[tracing::instrument(name = "get_config", level = "debug", skip(self))]
    async fn config_get(
        &mut self,
        room: SignalingRoomId,
    ) -> Result<Option<StorageConfig>, SignalingModuleError> {
        self.get(RoomAutomodConfig { room })
            .await
            .context(RedisSnafu {
                message: "Failed to get config",
            })
    }

    /// Delete the config.
    #[tracing::instrument(name = "del_config", level = "debug", skip(self))]
    async fn config_delete(&mut self, room: SignalingRoomId) -> Result<(), SignalingModuleError> {
        self.del(RoomAutomodConfig { room })
            .await
            .context(RedisSnafu {
                message: "Failed to del config",
            })
    }

    /// Query for the current config, if any is set.
    ///
    /// If it returns `true`, one can assume the automod is active.
    #[tracing::instrument(name = "exists_config", level = "debug", skip(self))]
    async fn config_exists(&mut self, room: SignalingRoomId) -> Result<bool, SignalingModuleError> {
        self.exists(RoomAutomodConfig { room })
            .await
            .context(RedisSnafu {
                message: "Failed to query if config exists",
            })
    }
}
/// Typed key to the automod config
#[derive(ToRedisArgs)]
#[to_redis_args(fmt = "opentalk-signaling:room={room}:automod:config")]
pub struct RoomAutomodConfig {
    room: SignalingRoomId,
}
