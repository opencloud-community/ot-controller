// SPDX-FileCopyrightText: OpenTalk GmbH <mail@opentalk.eu>
//
// SPDX-License-Identifier: EUPL-1.2

use derive_more::{AsRef, Display, From, FromStr, Into};
use uuid::Uuid;

#[allow(unused_imports)]
use crate::imports::*;

/// Unique id of a participant inside a single room
///
/// Generated as soon as the user connects to the websocket and authenticated himself,
/// it is used to store all participant related data and relations.
#[derive(
    AsRef, Debug, Copy, Clone, Ord, PartialOrd, Eq, PartialEq, Hash, Display, Into, From, FromStr,
)]
#[cfg_attr(
    feature = "redis",
    derive(FromRedisValue, ToRedisArgs),
    from_redis_value(FromStr),
    to_redis_args(fmt)
)]
#[cfg_attr(feature = "serde", derive(Serialize, Deserialize))]
pub struct ParticipantId(Uuid);

impl ParticipantId {
    /// Create a ZERO participant id, e.g. for testing purposes
    pub const fn nil() -> Self {
        Self(Uuid::nil())
    }

    /// Create a participant id from a number for testing purposes
    pub const fn from_u128(id: u128) -> Self {
        Self(Uuid::from_u128(id))
    }

    /// Generate a new random participant id
    #[cfg(feature = "rand")]
    pub fn generate() -> Self {
        Self(Uuid::new_v4())
    }
}
