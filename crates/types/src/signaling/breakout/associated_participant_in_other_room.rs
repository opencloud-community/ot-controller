// SPDX-FileCopyrightText: OpenTalk GmbH <mail@opentalk.eu>
//
// SPDX-License-Identifier: EUPL-1.2

use crate::core::{BreakoutRoomId, ParticipantId};

#[allow(unused_imports)]
use crate::imports::*;

/// Information about an associated participant in another breakout room
#[derive(Debug, Clone, PartialEq, Eq)]
#[cfg_attr(feature = "serde", derive(Serialize, Deserialize))]
pub struct AssociatedParticipantInOtherRoom {
    /// The id of the breakout room
    pub breakout_room: Option<BreakoutRoomId>,

    /// The id of the other participant
    pub id: ParticipantId,
}
