// SPDX-FileCopyrightText: OpenTalk GmbH <mail@opentalk.eu>
//
// SPDX-License-Identifier: EUPL-1.2

use derive_more::{AsRef, Display, From, FromStr, Into};
use uuid::Uuid;

use crate::utils::ExampleData;

/// ID of a streaming target
#[derive(
    AsRef, Display, From, FromStr, Into, Debug, Clone, Copy, PartialEq, Eq, Hash, PartialOrd, Ord,
)]
#[cfg_attr(
    feature = "redis",
    derive(redis_args::ToRedisArgs, redis_args::FromRedisValue)
)]
#[cfg_attr(feature = "redis", to_redis_args(fmt), from_redis_value(FromStr))]
#[cfg_attr(
    feature = "diesel",
    derive(
        opentalk_diesel_newtype::DieselNewtype,
        diesel::expression::AsExpression,
        diesel::deserialize::FromSqlRow
    )
)]
#[cfg_attr(feature="diesel",
    diesel(sql_type = diesel::sql_types::Uuid),
)]
#[cfg_attr(feature = "serde", derive(serde::Serialize, serde::Deserialize))]
#[cfg_attr(
    feature = "utoipa",
    derive(utoipa::ToSchema),
    schema(example = json!(StreamingTargetId::example_data()))
)]
pub struct StreamingTargetId(Uuid);

impl StreamingTargetId {
    /// Create a ZERO streaming target id, e.g. for testing purposes
    pub const fn nil() -> Self {
        Self(Uuid::nil())
    }

    /// Create a streaming target id from a number, e.g. for testing purposes
    pub const fn from_u128(id: u128) -> Self {
        Self(Uuid::from_u128(id))
    }

    /// Generate a new random streaming target id
    #[cfg(feature = "rand")]
    pub fn generate() -> Self {
        Self::from(Uuid::new_v4())
    }
}

impl ExampleData for StreamingTargetId {
    fn example_data() -> Self {
        Self::from_u128(0x43434343)
    }
}
