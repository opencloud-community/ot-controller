// SPDX-FileCopyrightText: OpenTalk GmbH <mail@opentalk.eu>
//
// SPDX-License-Identifier: EUPL-1.2

use std::time::Duration;

use async_trait::async_trait;
use opentalk_signaling_core::{RedisConnection, RedisSnafu, SignalingModuleError};
use opentalk_types::core::RoomId;
use redis::AsyncCommands as _;
use redis_args::ToRedisArgs;
use snafu::ResultExt as _;

use super::{BreakoutConfig, BreakoutStorage};

#[async_trait(?Send)]
impl BreakoutStorage for RedisConnection {
    #[tracing::instrument(level = "debug", skip(self))]
    async fn set_breakout_config(
        &mut self,
        room: RoomId,
        config: &BreakoutConfig,
    ) -> Result<Option<Duration>, SignalingModuleError> {
        if let Some(duration) = config.duration {
            let expiry_seconds = std::cmp::max(duration.as_secs(), 1);
            let expiry_duration = Duration::from_secs(expiry_seconds);
            self.set_ex(BreakoutRoomConfig { room }, config, expiry_seconds)
                .await
                .context(RedisSnafu {
                    message: "Failed to SET EX breakout-room config",
                })?;
            Ok(Some(expiry_duration))
        } else {
            self.set(BreakoutRoomConfig { room }, config)
                .await
                .context(RedisSnafu {
                    message: "Failed to SET breakout-room config",
                })?;
            Ok(None)
        }
    }

    #[tracing::instrument(level = "debug", skip(self))]
    async fn get_breakout_config(
        &mut self,
        room: RoomId,
    ) -> Result<Option<BreakoutConfig>, SignalingModuleError> {
        self.get(BreakoutRoomConfig { room })
            .await
            .context(RedisSnafu {
                message: "Failed to get breakout-room config",
            })
    }

    #[tracing::instrument(level = "debug", skip(self))]
    async fn del_breakout_config(&mut self, room: RoomId) -> Result<bool, SignalingModuleError> {
        self.del(BreakoutRoomConfig { room })
            .await
            .context(RedisSnafu {
                message: "Failed to del breakout-room config",
            })
    }
}

/// Typed key to the breakout-room config for the specified room
#[derive(ToRedisArgs)]
#[to_redis_args(fmt = "opentalk-signaling:room={room}:breakout:config")]
struct BreakoutRoomConfig {
    room: RoomId,
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

        redis::cmd("FLUSHALL").exec_async(&mut mgr).await.unwrap();

        RedisConnection::new(mgr)
    }

    #[tokio::test]
    #[serial]
    async fn config_unlimited() {
        test_common::config_unlimited(&mut storage().await).await;
    }

    #[tokio::test]
    #[serial]
    async fn config_expiring() {
        test_common::config_expiring(&mut storage().await).await;
    }
}
