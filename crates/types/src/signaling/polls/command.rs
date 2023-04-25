// SPDX-FileCopyrightText: OpenTalk GmbH <mail@opentalk.eu>
//
// SPDX-License-Identifier: EUPL-1.2

//! Signaling messages for the `polls` namespace

use std::time::Duration;

use crate::imports::*;

use super::{ChoiceId, PollId};

/// Command to start a poll
#[derive(Clone, Debug, PartialEq, Eq)]
#[cfg_attr(feature = "serde", derive(Serialize, Deserialize))]
pub struct Start {
    /// The description of the poll topic
    pub topic: String,

    /// Is the poll live
    pub live: bool,

    /// The choices of the poll
    pub choices: Vec<String>,

    /// The duration of the poll
    #[cfg_attr(feature = "serde", serde(with = "crate::utils::duration_seconds"))]
    pub duration: Duration,
}

/// Command to vote in the poll
#[derive(Clone, Debug, PartialEq, Eq)]
#[cfg_attr(feature = "serde", derive(Serialize, Deserialize))]
pub struct Vote {
    /// The id of the poll
    pub poll_id: PollId,

    /// The id of the choice
    pub choice_id: ChoiceId,
}

/// Command to finish the poll
#[derive(Clone, Debug, PartialEq, Eq)]
#[cfg_attr(feature = "serde", derive(Serialize, Deserialize))]
pub struct Finish {
    /// The id of the poll
    pub id: PollId,
}
