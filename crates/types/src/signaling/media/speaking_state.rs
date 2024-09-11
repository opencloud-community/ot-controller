// SPDX-FileCopyrightText: OpenTalk GmbH <mail@opentalk.eu>
//
// SPDX-License-Identifier: EUPL-1.2

use opentalk_types_common::time::Timestamp;

#[allow(unused_imports)]
use crate::imports::*;

/// The state of a recent or current speaker in the conference
#[derive(Clone, Debug, Default, PartialEq, Eq)]
#[cfg_attr(
    feature = "redis",
    derive(ToRedisArgs, FromRedisValue),
    to_redis_args(serde),
    from_redis_value(serde)
)]
#[cfg_attr(feature = "serde", derive(Serialize, Deserialize))]
pub struct SpeakingState {
    /// A flag indicating whether the speaker is currently speaking
    pub is_speaking: bool,

    /// The timestamp when the speaker state was last updated
    pub updated_at: Timestamp,
}
