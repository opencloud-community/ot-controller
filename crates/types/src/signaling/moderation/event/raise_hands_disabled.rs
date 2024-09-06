// SPDX-FileCopyrightText: OpenTalk GmbH <mail@opentalk.eu>
//
// SPDX-License-Identifier: EUPL-1.2

use opentalk_types_signaling::ParticipantId;

#[allow(unused_imports)]
use crate::imports::*;

/// Sent out when raise hands is disabled
#[derive(Clone, Debug, PartialEq, Eq)]
#[cfg_attr(feature = "serde", derive(Serialize, Deserialize))]
pub struct RaiseHandsDisabled {
    /// The moderator who disabled raise hands
    pub issued_by: ParticipantId,
}
