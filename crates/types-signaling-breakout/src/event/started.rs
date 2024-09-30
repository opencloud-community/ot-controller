// SPDX-FileCopyrightText: OpenTalk GmbH <mail@opentalk.eu>
//
// SPDX-License-Identifier: EUPL-1.2

use opentalk_types_common::{rooms::BreakoutRoomId, time::Timestamp};

use crate::BreakoutRoom;

/// Event signaling to the participant that the breakout session has started
#[derive(Clone, Debug, PartialEq, Eq)]
#[cfg_attr(feature = "serde", derive(serde::Serialize, serde::Deserialize))]
pub struct Started {
    /// List of the breakout rooms
    pub rooms: Vec<BreakoutRoom>,

    /// The expiration time of the breakout session
    pub expires: Option<Timestamp>,

    /// The id of the assigned breakout room
    pub assignment: Option<BreakoutRoomId>,
}
