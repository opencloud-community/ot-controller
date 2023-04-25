// SPDX-FileCopyrightText: OpenTalk GmbH <mail@opentalk.eu>
//
// SPDX-License-Identifier: EUPL-1.2

//! Signaling messages for the `polls` namespace

use crate::imports::*;

use super::PollId;

/// Command to finish the poll
#[derive(Clone, Debug, PartialEq, Eq)]
#[cfg_attr(feature = "serde", derive(Serialize, Deserialize))]
pub struct Finish {
    /// The id of the poll
    pub id: PollId,
}
