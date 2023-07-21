// SPDX-FileCopyrightText: OpenTalk GmbH <mail@opentalk.eu>
//
// SPDX-License-Identifier: EUPL-1.2

//! Signaling commands for the `chat` namespace

#[allow(unused_imports)]
use crate::imports::*;

/// The different timer variations
#[derive(Clone, Copy, PartialEq, Eq, Debug)]
#[cfg_attr(
    feature = "serde",
    derive(Deserialize),
    serde(rename_all = "snake_case", tag = "kind")
)]
pub enum Kind {
    /// The timer continues to run until a moderator stops it.
    Stopwatch,
    /// The timer continues to run until its duration expires or if a moderator stops it beforehand.
    Countdown {
        /// The duration of the countdown
        duration: u64,
    },
}
