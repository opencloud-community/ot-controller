// SPDX-FileCopyrightText: OpenTalk GmbH <mail@opentalk.eu>
//
// SPDX-License-Identifier: EUPL-1.2

use async_trait::async_trait;
use opentalk_signaling_core::RedisConnection;

use super::AutomodStorage;

pub mod allow_list;
pub mod config;
pub mod history;
pub mod playlist;
pub mod speaker;

#[async_trait(?Send)]
impl AutomodStorage for RedisConnection {}

#[cfg(test)]
mod tests {
    use opentalk_signaling_core::RedisConnection;
    use redis::aio::ConnectionManager;
    use serial_test::serial;

    use crate::storage::test_common;

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
    async fn playlist() {
        test_common::playlist(&mut storage().await).await;
    }

    #[tokio::test]
    #[serial]
    async fn playlist_remove_first() {
        test_common::playlist_remove_first(&mut storage().await).await
    }

    #[tokio::test]
    #[serial]
    async fn allow_list() {
        test_common::allow_list(&mut storage().await).await
    }

    #[tokio::test]
    #[serial]
    async fn storage_config() {
        test_common::storage_config(&mut storage().await).await
    }

    #[tokio::test]
    #[serial]
    async fn speaker() {
        test_common::speaker(&mut storage().await).await
    }

    #[tokio::test]
    #[serial]
    async fn history() {
        test_common::history(&mut storage().await).await
    }

    #[tokio::test]
    #[serial]
    async fn history_repeated_speaker() {
        test_common::history_repeated_speaker(&mut storage().await).await
    }
}
