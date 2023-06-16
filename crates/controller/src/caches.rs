// SPDX-FileCopyrightText: OpenTalk GmbH <mail@opentalk.eu>
//
// SPDX-License-Identifier: EUPL-1.2

use signaling_core::RedisConnection;

use crate::api::v1::middleware::user_auth::UserAccessTokenCache;
use core::time::Duration;

/// Holds all application level caches
pub struct Caches {
    /// Cache the results of user access-token checks
    pub user_access_tokens: UserAccessTokenCache,
}

impl Caches {
    pub fn create(redis: RedisConnection) -> Self {
        let redis = redis.into_manager();

        Self {
            user_access_tokens: UserAccessTokenCache::new(Duration::from_secs(300)).with_redis(
                redis,
                "user-access-tokens",
                Duration::from_secs(300),
                true,
            ),
        }
    }
}
