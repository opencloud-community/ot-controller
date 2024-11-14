// SPDX-FileCopyrightText: OpenTalk GmbH <mail@opentalk.eu>
//
// SPDX-License-Identifier: EUPL-1.2

use core::time::Duration;

use opentalk_signaling_core::RedisConnection;

use crate::api::v1::middleware::user_auth::UserAccessTokenCache;

/// Holds all application level caches
pub struct Caches {
    /// Cache the results of user access-token checks
    pub user_access_tokens: UserAccessTokenCache,
}

impl Caches {
    pub fn create(redis: Option<RedisConnection>) -> Self {
        let mut user_access_tokens = UserAccessTokenCache::new(Duration::from_secs(300));

        if let Some(redis) = redis {
            let redis = redis.into_manager();

            user_access_tokens = user_access_tokens.with_redis(
                redis,
                "user-access-tokens",
                Duration::from_secs(300),
                true,
            )
        };

        Self { user_access_tokens }
    }
}
