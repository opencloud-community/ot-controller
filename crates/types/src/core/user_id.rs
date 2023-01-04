// SPDX-FileCopyrightText: OpenTalk GmbH <mail@opentalk.eu>
//
// SPDX-License-Identifier: EUPL-1.2

use derive_more::{AsRef, Display, From, FromStr, Into};
use uuid::Uuid;

#[allow(unused_imports)]
use crate::imports::*;

#[cfg(feature = "kustos")]
use kustos_shared::subject::PolicyUser;

/// The id of a user
#[derive(
    AsRef, Display, From, FromStr, Into, Debug, Clone, Copy, PartialEq, Eq, PartialOrd, Ord, Hash,
)]
#[cfg_attr(feature="diesel",
    derive(DieselNewtype, AsExpression, FromSqlRow),
    diesel(sql_type = diesel::sql_types::Uuid),
)]
#[cfg_attr(feature = "kustos", derive(KustosPrefix), kustos_prefix("/users/"))]
#[cfg_attr(
    feature = "redis",
    derive(ToRedisArgs, FromRedisValue),
    to_redis_args(fmt),
    from_redis_value(FromStr)
)]
#[cfg_attr(feature = "serde", derive(Serialize, Deserialize))]
pub struct UserId(Uuid);

impl UserId {
    /// Create a ZERO user id, e.g. for testing purposes
    pub const fn nil() -> Self {
        Self(Uuid::nil())
    }

    /// Create a user id from a number, e.g. for testing purposes
    pub const fn from_u128(id: u128) -> Self {
        Self(Uuid::from_u128(id))
    }

    /// Generate a new random user id
    #[cfg(feature = "rand")]
    pub fn generate() -> Self {
        Self(Uuid::new_v4())
    }
}

#[cfg(feature = "kustos")]
impl From<UserId> for PolicyUser {
    fn from(id: UserId) -> Self {
        Self::from(Uuid::from(id))
    }
}
