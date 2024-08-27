// SPDX-FileCopyrightText: OpenTalk GmbH <mail@opentalk.eu>
//
// SPDX-License-Identifier: EUPL-1.2

use derive_more::{AsRef, Display, From, FromStr, Into};

#[allow(unused_imports)]
use crate::imports::*;
use crate::{call_in::NumericId, utils::ExampleData};

/// The id of a call-in participation
#[derive(
    AsRef, Display, From, FromStr, Into, Debug, Clone, PartialEq, Eq, PartialOrd, Ord, Hash,
)]
#[cfg_attr(
    feature = "diesel",
    derive(DieselNewtype, AsExpression, FromSqlRow),
    diesel(sql_type = diesel::sql_types::Text)
)]
#[cfg_attr(
    feature = "redis",
    derive(ToRedisArgs, FromRedisValue),
    to_redis_args(fmt),
    from_redis_value(FromStr)
)]
#[cfg_attr(feature = "serde", derive(Serialize, Deserialize))]
#[cfg_attr(feature = "utoipa", derive(utoipa::ToSchema), schema(example = json!(CallInId::example_data())))]
pub struct CallInId(NumericId);

impl CallInId {
    /// Generate a random sip id
    #[cfg(feature = "rand")]
    pub fn generate() -> Self {
        Self::from(NumericId::generate())
    }
}

impl ExampleData for CallInId {
    fn example_data() -> Self {
        Self("0123456789".parse().expect("parseable numeric id"))
    }
}
