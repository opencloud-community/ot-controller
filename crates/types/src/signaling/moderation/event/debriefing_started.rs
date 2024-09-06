// SPDX-FileCopyrightText: OpenTalk GmbH <mail@opentalk.eu>
//
// SPDX-License-Identifier: EUPL-1.2

use opentalk_types_signaling::ParticipantId;

#[allow(unused_imports)]
use crate::imports::*;

/// Sent out when debriefing of a session started
#[derive(Clone, Debug, PartialEq, Eq)]
#[cfg_attr(feature = "serde", derive(Serialize, Deserialize))]
pub struct DebriefingStarted {
    /// The moderator who started the debriefing
    pub issued_by: ParticipantId,
}
