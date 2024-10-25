// SPDX-FileCopyrightText: OpenTalk GmbH <mail@opentalk.eu>
//
// SPDX-License-Identifier: EUPL-1.2

use crate::ChoiceId;

/// Represents the polling count for a poll choice
#[derive(Debug, Clone, PartialEq, Eq)]
#[cfg_attr(feature = "serde", derive(serde::Serialize, serde::Deserialize))]
pub struct Item {
    /// The id of the choice
    pub id: ChoiceId,

    /// The polling count for the choice
    pub count: u32,
}
