// SPDX-FileCopyrightText: OpenTalk GmbH <mail@opentalk.eu>
//
// SPDX-License-Identifier: EUPL-1.2

use crate::core::RoomId;

use super::CallInInfo;

#[allow(unused_imports)]
use crate::imports::*;

/// All information about a room in which an event takes place
#[derive(Debug, Clone, PartialEq, Eq)]
#[cfg_attr(feature = "serde", derive(Serialize, Deserialize))]
pub struct EventRoomInfo {
    /// ID of the room
    pub id: RoomId,

    /// Password of the room
    #[cfg_attr(feature = "serde", serde(skip_serializing_if = "Option::is_none"))]
    pub password: Option<String>,

    /// Flag to check if the room has a waiting room enabled
    pub waiting_room: bool,

    /// Call-In information
    #[cfg_attr(feature = "serde", serde(skip_serializing_if = "Option::is_none"))]
    pub call_in: Option<CallInInfo>,
}
