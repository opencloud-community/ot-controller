// SPDX-FileCopyrightText: OpenTalk GmbH <mail@opentalk.eu>
//
// SPDX-License-Identifier: EUPL-1.2

use async_trait::async_trait;
use opentalk_signaling_core::{RedisConnection, RedisSnafu, SignalingModuleError, SignalingRoomId};
use redis::AsyncCommands;
use redis_args::ToRedisArgs;
use snafu::ResultExt;

use super::{InitState, SpaceInfo, WhiteboardStorage};

#[async_trait(?Send)]
impl WhiteboardStorage for RedisConnection {
    #[tracing::instrument(name = "spacedeck_try_start_init", skip(self))]
    async fn try_start_init(
        &mut self,
        room_id: SignalingRoomId,
    ) -> Result<Option<InitState>, SignalingModuleError> {
        let affected_entries: i64 = self
            .set_nx(InitStateKey { room_id }, InitState::Initializing)
            .await
            .context(RedisSnafu {
                message: "Failed to set spacedeck init state",
            })?;

        if affected_entries == 1 {
            Ok(None)
        } else {
            let state: InitState =
                self.get(InitStateKey { room_id })
                    .await
                    .context(RedisSnafu {
                        message: "Failed to get spacedeck init state",
                    })?;

            Ok(Some(state))
        }
    }

    /// Sets the room state to [`InitState::Initialized(..)`]
    #[tracing::instrument(name = "spacedeck_set_initialized", skip(self, space_info))]
    async fn set_initialized(
        &mut self,
        room_id: SignalingRoomId,
        space_info: SpaceInfo,
    ) -> Result<(), SignalingModuleError> {
        let initialized = InitState::Initialized(space_info);

        self.set(InitStateKey { room_id }, &initialized)
            .await
            .context(RedisSnafu {
                message: "Failed to set spacedeck init state to `Initialized",
            })
    }

    #[tracing::instrument(name = "get_spacedeck_init_state", skip(self))]
    async fn get_init_state(
        &mut self,
        room_id: SignalingRoomId,
    ) -> Result<Option<InitState>, SignalingModuleError> {
        self.get(InitStateKey { room_id })
            .await
            .context(RedisSnafu {
                message: "Failed to get spacedeck init state",
            })
    }

    #[tracing::instrument(name = "delete_spacedeck_init_state", skip(self))]
    async fn delete_init_state(
        &mut self,
        room_id: SignalingRoomId,
    ) -> Result<(), SignalingModuleError> {
        self.del::<_, i64>(InitStateKey { room_id })
            .await
            .context(RedisSnafu {
                message: "Failed to delete spacedeck init key",
            })?;

        Ok(())
    }
}

/// Stores the [`InitState`] of this room.
#[derive(ToRedisArgs)]
#[to_redis_args(fmt = "opentalk-signaling:room={room_id}:spacedeck:init")]
pub(super) struct InitStateKey {
    pub(super) room_id: SignalingRoomId,
}

#[cfg(test)]
mod tests {
    use redis::aio::ConnectionManager;
    use serial_test::serial;

    use super::{super::test_common, *};

    async fn storage() -> RedisConnection {
        let redis_url =
            std::env::var("REDIS_ADDR").unwrap_or_else(|_| "redis://0.0.0.0:6379/".to_owned());
        let redis = redis::Client::open(redis_url).expect("Invalid redis url");

        let mut mgr = ConnectionManager::new(redis).await.unwrap();

        redis::cmd("FLUSHALL").exec_async(&mut mgr).await.unwrap();

        RedisConnection::new(mgr)
    }

    #[tokio::test]
    #[serial]
    async fn initialization() {
        test_common::initialization(&mut storage().await).await;
    }
}
