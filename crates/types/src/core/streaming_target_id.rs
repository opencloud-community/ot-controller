// SPDX-FileCopyrightText: OpenTalk GmbH <mail@opentalk.eu>
//
// SPDX-License-Identifier: EUPL-1.2

use uuid::Uuid;

#[allow(unused_imports)]
use crate::imports::*;

crate::diesel_newtype! {
    feature_gated:

    #[derive(
        derive_more::AsRef,
        derive_more::From,
        derive_more::Into,
        derive_more::FromStr,
        Copy,
    )]
    #[cfg_attr(
        feature = "redis",
        derive(redis_args::ToRedisArgs, redis_args::FromRedisValue),
        to_redis_args(fmt),
        from_redis_value(FromStr)
    )]
    StreamingTargetId(uuid::Uuid) => diesel::sql_types::Uuid
}

impl StreamingTargetId {
    /// Create a ZERO streaming target id, e.g. for testing purposes
    pub const fn nil() -> Self {
        Self::from(Uuid::nil())
    }

    /// Create a streaming target id from a number, e.g. for testing purposes
    pub const fn from_u128(id: u128) -> Self {
        Self::from(Uuid::from_u128(id))
    }

    /// Generate a new random streaming target id
    #[cfg(feature = "rand")]
    pub fn generate() -> Self {
        Self::from(Uuid::new_v4())
    }
}
