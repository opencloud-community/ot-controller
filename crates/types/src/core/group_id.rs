// SPDX-FileCopyrightText: OpenTalk GmbH <mail@opentalk.eu>
//
// SPDX-License-Identifier: EUPL-1.2

use uuid::Uuid;

crate::diesel_newtype! {
    feature_gated:

    #[derive(Copy, derive_more::FromStr)]
    #[cfg_attr(
        feature = "redis",
        derive(redis_args::ToRedisArgs, redis_args::FromRedisValue),
        to_redis_args(fmt),
        from_redis_value(FromStr)
    )]
    GroupId(uuid::Uuid) => diesel::sql_types::Uuid
}

impl GroupId {
    /// Create a ZERO group id, e.g. for testing purposes
    pub const fn nil() -> Self {
        Self::from(Uuid::nil())
    }

    /// Create a group id from a number, e.g. for testing purposes
    pub const fn from_u128(id: u128) -> Self {
        Self::from(Uuid::from_u128(id))
    }

    /// Generate a new random group id
    #[cfg(feature = "rand")]
    pub fn generate() -> Self {
        Self::from(Uuid::new_v4())
    }
}

#[cfg(feature = "kustos")]
impl From<GroupId> for kustos::subject::PolicyGroup {
    fn from(group_id: GroupId) -> Self {
        Self::from(group_id.to_string())
    }
}
