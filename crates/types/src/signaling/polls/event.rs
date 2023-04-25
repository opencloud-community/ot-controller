// SPDX-FileCopyrightText: OpenTalk GmbH <mail@opentalk.eu>
//
// SPDX-License-Identifier: EUPL-1.2

//! Types related to signaling events in the `polls` namespace

use std::time::Duration;

use crate::imports::*;

use super::{Choice, PollId};

/// Event signaling to the participant that the poll has started
#[derive(Clone, Debug, PartialEq, Eq)]
#[cfg_attr(feature = "serde", derive(Serialize, Deserialize))]
pub struct Started {
    /// The id of the poll
    pub id: PollId,

    /// The description of the poll topic
    pub topic: String,

    /// Is the poll live
    pub live: bool,

    /// Choices of the poll
    pub choices: Vec<Choice>,

    /// Duration of the poll
    #[cfg_attr(feature = "serde", serde(with = "crate::utils::duration_seconds"))]
    pub duration: Duration,
}

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
