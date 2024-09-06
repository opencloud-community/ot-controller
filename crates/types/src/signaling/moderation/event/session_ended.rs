// SPDX-FileCopyrightText: OpenTalk GmbH <mail@opentalk.eu>
//
// SPDX-License-Identifier: EUPL-1.2

use opentalk_types_signaling::ParticipantId;

#[allow(unused_imports)]
use crate::imports::*;

/// Sent out when a session is ended by a moderator
#[derive(Clone, Debug, PartialEq, Eq)]
#[cfg_attr(feature = "serde", derive(Serialize, Deserialize))]
pub struct SessionEnded {
    /// The moderator who ended the session
    pub issued_by: ParticipantId,
}
