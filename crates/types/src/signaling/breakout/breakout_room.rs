// SPDX-FileCopyrightText: OpenTalk GmbH <mail@opentalk.eu>
//
// SPDX-License-Identifier: EUPL-1.2

use crate::core::BreakoutRoomId;

#[allow(unused_imports)]
use crate::imports::*;

/// Description of a breakout room
#[derive(Debug, Clone, PartialEq, Eq)]
#[cfg_attr(feature = "serde", derive(Serialize, Deserialize))]
pub struct BreakoutRoom {
    /// The id of the breakout room
    pub id: BreakoutRoomId,
    /// The name of the breakout room
    pub name: String,
}
