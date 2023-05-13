// SPDX-FileCopyrightText: OpenTalk GmbH <mail@opentalk.eu>
//
// SPDX-License-Identifier: EUPL-1.2

//! Types related to signaling events in the `polls` namespace

use crate::imports::*;

/// Errors from the `polls` module namespace
#[derive(Clone, Debug, PartialEq, Eq)]
#[cfg_attr(
    feature = "serde",
    derive(Serialize, Deserialize),
    serde(rename_all = "snake_case", tag = "error")
)]
pub enum Error {
    /// Attempted to perform a command which requires more permissions
    InsufficientPermissions,

    /// Attempted to start a poll with invalid choice count
    InvalidChoiceCount,

    /// Attempted to perform a command with an invalid poll id
    InvalidPollId,

    /// Attempted to perform a command with an invalid choice id
    InvalidChoiceId,

    /// Attempted to perform a command with an invalid choice description
    InvalidChoiceDescription,

    /// Attempted to perform a command with an invalid duration
    InvalidDuration,

    /// Attempted to perform a command with an invalid topic length
    InvalidTopicLength,

    /// Attempted to vote again
    VotedAlready,

    /// Attempted to start a new poll while an existing one is still running
    StillRunning,
}
