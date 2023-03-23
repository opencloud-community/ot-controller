// SPDX-FileCopyrightText: OpenTalk GmbH <mail@opentalk.eu>
//
// SPDX-License-Identifier: EUPL-1.2

#[cfg(feature = "kustos")]
use kustos_shared::subject::PolicyUser;

use uuid::Uuid;

crate::diesel_newtype! {
    feature_gated:

    #[derive(Copy)]
    // If feature `kustos` is enabled, `FromStr` is implemented by the
    // `diesel_newtype!(…)` macro.
    #[cfg_attr(
        not(feature = "kustos"),
        derive(derive_more::FromStr),
    )]
    #[cfg_attr(
        feature = "redis",
        derive(redis_args::ToRedisArgs, redis_args::FromRedisValue),
        to_redis_args(fmt),
        from_redis_value(FromStr)
    )]
    UserId(uuid::Uuid) => diesel::sql_types::Uuid, "/users/"
}

impl UserId {
    /// Create a ZERO user id, e.g. for testing purposes
    pub const fn nil() -> Self {
        Self::from(Uuid::nil())
    }

    /// Create a user id from a number, e.g. for testing purposes
    pub const fn from_u128(id: u128) -> Self {
        Self::from(Uuid::from_u128(id))
    }

    /// Generate a new random user id
    #[cfg(feature = "rand")]
    pub fn generate() -> Self {
        Self::from(Uuid::new_v4())
    }
}

#[cfg(feature = "kustos")]
impl From<UserId> for PolicyUser {
    fn from(id: UserId) -> Self {
        Self::from(id.into_inner())
    }
}
