// SPDX-FileCopyrightText: OpenTalk GmbH <mail@opentalk.eu>
//
// SPDX-License-Identifier: EUPL-1.2

use crate::core::Timestamp;
#[allow(unused_imports)]
use crate::imports::*;

/// The different timer variations
#[derive(Debug, Clone, Copy, PartialEq, Eq)]
#[cfg_attr(
    feature = "serde",
    derive(Serialize, Deserialize),
    serde(rename_all = "snake_case", tag = "kind")
)]
pub enum Kind {
    /// The timer continues to run until a moderator stops it.
    Stopwatch,
    /// The timer continues to run until its duration expires or if a moderator stops it beforehand.
    Countdown {
        /// When the timer will end
        ends_at: Timestamp,
    },
}
