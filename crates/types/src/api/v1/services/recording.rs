// SPDX-FileCopyrightText: OpenTalk GmbH <mail@opentalk.eu>
//
// SPDX-License-Identifier: EUPL-1.2

use crate::core::{BreakoutRoomId, RoomId};
#[allow(unused_imports)]
use crate::imports::*;

/// Response for the `POST /services/recording/start` endpoint
#[derive(Debug, Clone, Eq, PartialEq)]
#[cfg_attr(feature = "serde", derive(Serialize, Deserialize))]
pub struct StartBody {
    /// The room id
    pub room_id: RoomId,

    /// The optional breakout room id
    pub breakout_room: Option<BreakoutRoomId>,
}
