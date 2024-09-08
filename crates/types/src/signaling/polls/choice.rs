// SPDX-FileCopyrightText: OpenTalk GmbH <mail@opentalk.eu>
//
// SPDX-License-Identifier: EUPL-1.2

use opentalk_types_signaling_polls::ChoiceId;

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
