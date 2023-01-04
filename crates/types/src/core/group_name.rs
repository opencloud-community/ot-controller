// SPDX-FileCopyrightText: OpenTalk GmbH <mail@opentalk.eu>
//
// SPDX-License-Identifier: EUPL-1.2

use derive_more::{AsRef, Display, From, FromStr, Into};

#[allow(unused_imports)]
use crate::imports::*;

/// The name of a group
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
    derive(ToRedisArgs, FromRedisValue,),
    to_redis_args(fmt),
    from_redis_value(FromStr)
)]
#[cfg_attr(feature = "serde", derive(Serialize, Deserialize))]
pub struct GroupName(String);
