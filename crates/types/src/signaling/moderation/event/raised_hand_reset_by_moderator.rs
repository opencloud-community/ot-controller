// SPDX-FileCopyrightText: OpenTalk GmbH <mail@opentalk.eu>
//
// SPDX-License-Identifier: EUPL-1.2

use opentalk_types_signaling::ParticipantId;

#[allow(unused_imports)]
use crate::imports::*;

/// Sent out when raised hand is reset by a moderator
#[derive(Clone, Debug, PartialEq, Eq)]
#[cfg_attr(feature = "serde", derive(Serialize, Deserialize))]
pub struct RaisedHandResetByModerator {
    /// The moderator who reset raised hand
    pub issued_by: ParticipantId,
}
