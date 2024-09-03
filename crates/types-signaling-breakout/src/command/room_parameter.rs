// SPDX-FileCopyrightText: OpenTalk GmbH <mail@opentalk.eu>
//
// SPDX-License-Identifier: EUPL-1.2

use opentalk_types_signaling::ParticipantId;

/// Parameters used for starting a breakout room
#[derive(Clone, Debug, PartialEq, Eq)]
#[cfg_attr(feature = "serde", derive(serde::Serialize, serde::Deserialize))]
pub struct RoomParameter {
    /// Name of the breakout room
    pub name: String,

    /// Ids of participants to be assigned to the breakout room
    pub assignments: Vec<ParticipantId>,
}
