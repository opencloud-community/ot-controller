// SPDX-FileCopyrightText: OpenTalk GmbH <mail@opentalk.eu>
//
// SPDX-License-Identifier: EUPL-1.2

use derive_more::{AsRef, Display, From, FromStr, Into};
use uuid::Uuid;

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
        PollId(Uuid::nil())
    }

    /// Generate a new random poll id
    #[cfg(feature = "rand")]
    pub fn generate() -> Self {
        PollId(Uuid::new_v4())
    }
}
