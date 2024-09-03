// SPDX-FileCopyrightText: OpenTalk GmbH <mail@opentalk.eu>
//
// SPDX-License-Identifier: EUPL-1.2

use opentalk_types_common::rooms::BreakoutRoomId;
use opentalk_types_signaling::ParticipantId;

/// Information about an associated participant in another breakout room
#[derive(Debug, Clone, PartialEq, Eq)]
#[cfg_attr(feature = "serde", derive(serde::Serialize, serde::Deserialize))]
pub struct AssociatedParticipantInOtherRoom {
    /// The id of the breakout room
    pub breakout_room: Option<BreakoutRoomId>,

    /// The id of the other participant
    pub id: ParticipantId,
}
