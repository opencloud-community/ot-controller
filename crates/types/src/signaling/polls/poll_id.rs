// SPDX-FileCopyrightText: OpenTalk GmbH <mail@opentalk.eu>
//
// SPDX-License-Identifier: EUPL-1.2

use derive_more::{AsRef, Display, From, FromStr, Into};
use uuid::Uuid;

#[allow(unused_imports)]
use crate::imports::*;

/// The id of the Poll
#[derive(Clone, Copy, Debug, PartialEq, Eq, Hash, FromStr, AsRef, Display, From, Into)]
#[cfg_attr(
    feature = "redis",
    derive(redis_args::ToRedisArgs, redis_args::FromRedisValue),
    to_redis_args(fmt),
    from_redis_value(FromStr)
)]
#[cfg_attr(feature = "serde", derive(Serialize, Deserialize))]
pub struct PollId(Uuid);

impl PollId {
    /// Create a ZERO poll id, e.g. for testing purposes
    pub const fn nil() -> Self {
        Self(Uuid::nil())
    }

    /// Create a poll id from a number, e.g. for testing purposes
    pub const fn from_u128(id: u128) -> Self {
        Self(Uuid::from_u128(id))
    }

    /// Generate a new random poll id
    #[cfg(feature = "rand")]
    pub fn generate() -> Self {
        Self(Uuid::new_v4())
    }
}
