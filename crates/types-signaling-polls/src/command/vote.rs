// SPDX-FileCopyrightText: OpenTalk GmbH <mail@opentalk.eu>
//
// SPDX-License-Identifier: EUPL-1.2

use super::Choices;
use crate::PollId;

/// Command to vote in the poll
#[derive(Clone, Debug, PartialEq, Eq)]
#[cfg_attr(feature = "serde", derive(serde::Serialize, serde::Deserialize))]
pub struct Vote {
    /// The id of the poll
    pub poll_id: PollId,

    /// The choices
    #[cfg_attr(feature = "serde", serde(flatten))]
    pub choices: Choices,
}
