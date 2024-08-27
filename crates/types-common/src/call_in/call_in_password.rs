// SPDX-FileCopyrightText: OpenTalk GmbH <mail@opentalk.eu>
//
// SPDX-License-Identifier: EUPL-1.2

use derive_more::{AsRef, Display, From, FromStr, Into};

#[allow(unused_imports)]
use crate::imports::*;
use crate::{call_in::NumericId, utils::ExampleData};

/// The password for authenticating call-in participation
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
#[cfg_attr(feature = "utoipa", derive(utoipa::ToSchema), schema(example = json!(CallInPassword::example_data())))]
pub struct CallInPassword(NumericId);

impl CallInPassword {
    /// Generate a random sip password
    #[cfg(feature = "rand")]
    pub fn generate() -> Self {
        Self::from(NumericId::generate())
    }
}

impl ExampleData for CallInPassword {
    fn example_data() -> Self {
        Self("9876543210".parse().expect("parseable numeric id"))
    }
}
