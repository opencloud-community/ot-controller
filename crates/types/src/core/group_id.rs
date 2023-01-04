// SPDX-FileCopyrightText: OpenTalk GmbH <mail@opentalk.eu>
//
// SPDX-License-Identifier: EUPL-1.2

use derive_more::{AsRef, Display, From, FromStr, Into};
use uuid::Uuid;

#[allow(unused_imports)]
use crate::imports::*;

/// The identifier of a group
#[derive(
    AsRef, Display, From, FromStr, Into, Debug, Clone, Copy, PartialEq, Eq, PartialOrd, Ord, Hash,
)]
#[cfg_attr(
    feature = "diesel",
    derive(DieselNewtype, AsExpression, FromSqlRow),
    diesel(sql_type = diesel::sql_types::Uuid),
)]
#[cfg_attr(feature = "kustos", derive(KustosPrefix), kustos_prefix("/events/"))]
#[cfg_attr(
    feature = "redis",
    derive(redis_args::ToRedisArgs, redis_args::FromRedisValue),
    to_redis_args(fmt),
    from_redis_value(FromStr)
)]
#[cfg_attr(feature = "serde", derive(Serialize, Deserialize))]
pub struct GroupId(Uuid);

impl GroupId {
    /// Create a ZERO group id, e.g. for testing purposes
    pub const fn nil() -> Self {
        Self(Uuid::nil())
    }

    /// Create a group id from a number, e.g. for testing purposes
    pub const fn from_u128(id: u128) -> Self {
        Self(Uuid::from_u128(id))
    }

    /// Generate a new random group id
    #[cfg(feature = "rand")]
    pub fn generate() -> Self {
        Self(Uuid::new_v4())
    }
}

#[cfg(feature = "kustos")]
impl From<GroupId> for kustos_shared::subject::PolicyGroup {
    fn from(group_id: GroupId) -> Self {
        Self::from(group_id.to_string())
    }
}
