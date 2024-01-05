// SPDX-FileCopyrightText: OpenTalk GmbH <mail@opentalk.eu>
//
// SPDX-License-Identifier: EUPL-1.2

//! Frontend data for `polls` namespace

use std::time::Duration;

use chrono::Utc;

use crate::core::Timestamp;

use super::{Choice, PollId};

#[allow(unused_imports)]
use crate::imports::*;

/// The state of the `polls` module.
///
/// This struct is sent to the participant in the `join_success` message
/// when they join successfully to the meeting.
#[derive(Clone, Debug, PartialEq, Eq)]
#[cfg_attr(
    feature = "redis",
    derive(ToRedisArgs, FromRedisValue),
    to_redis_args(serde),
    from_redis_value(serde)
)]
#[cfg_attr(feature = "serde", derive(Serialize, Deserialize))]
pub struct PollsState {
    /// The id of the poll
    pub id: PollId,

    /// The description of the poll topic
    pub topic: String,

    /// Is the poll live
    pub live: bool,

    /// Choices of the poll
    pub choices: Vec<Choice>,

    /// The time when the poll started
    pub started: Timestamp,

    /// The duration of the poll
    #[cfg_attr(feature = "serde", serde(with = "crate::utils::duration_seconds"))]
    pub duration: Duration,
}

#[cfg(feature = "serde")]
impl SignalingModuleFrontendData for PollsState {
    const NAMESPACE: Option<&'static str> = Some(super::NAMESPACE);
}

impl PollsState {
    /// Get the remaining duration of the poll
    pub fn remaining(&self) -> Option<Duration> {
        let duration = chrono::Duration::from_std(self.duration)
            .expect("duration as secs should never be larger than i64::MAX");

        let expire = (*self.started) + duration;
        let now = Utc::now();

        // difference will be negative duration if expired.
        // Conversion to std duration will fail -> returning None
        (expire - now).to_std().ok()
    }

    /// Is the poll expired
    pub fn is_expired(&self) -> bool {
        self.remaining().is_none()
    }
}
