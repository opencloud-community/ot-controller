// SPDX-FileCopyrightText: OpenTalk GmbH <mail@opentalk.eu>
//
// SPDX-License-Identifier: EUPL-1.2

use super::ChoiceId;
#[allow(unused_imports)]
use crate::imports::*;

/// The choice for a poll
#[derive(Debug, Clone, PartialEq, Eq)]
#[cfg_attr(feature = "serde", derive(Serialize, Deserialize))]
pub struct Choice {
    /// The id of the choice
    pub id: ChoiceId,
    /// The content of the choice
    pub content: String,
}
