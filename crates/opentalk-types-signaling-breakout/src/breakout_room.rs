// SPDX-FileCopyrightText: OpenTalk GmbH <mail@opentalk.eu>
//
// SPDX-License-Identifier: EUPL-1.2

use opentalk_types_common::rooms::BreakoutRoomId;

/// Description of a breakout room
#[derive(Debug, Clone, PartialEq, Eq)]
#[cfg_attr(feature = "serde", derive(serde::Serialize, serde::Deserialize))]
pub struct BreakoutRoom {
    /// The id of the breakout room
    pub id: BreakoutRoomId,

    /// The name of the breakout room
    pub name: String,
}
