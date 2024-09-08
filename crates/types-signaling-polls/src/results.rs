// SPDX-FileCopyrightText: OpenTalk GmbH <mail@opentalk.eu>
//
// SPDX-License-Identifier: EUPL-1.2

use crate::{Item, PollId};

/// Represents the results of a completed poll
#[derive(Debug, Clone, PartialEq, Eq)]
#[cfg_attr(feature = "serde", derive(serde::Serialize, serde::Deserialize))]
pub struct Results {
    /// The id of the poll
    pub id: PollId,

    /// The poll items with their counts
    pub results: Vec<Item>,
}
