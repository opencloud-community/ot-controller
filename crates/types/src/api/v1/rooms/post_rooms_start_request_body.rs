// SPDX-FileCopyrightText: OpenTalk GmbH <mail@opentalk.eu>
//
// SPDX-License-Identifier: EUPL-1.2

use crate::core::{BreakoutRoomId, ResumptionToken};

#[allow(unused_imports)]
use crate::imports::*;

/// The JSON body expected when making a *POST /rooms/{room_id}/start*
#[derive(Clone, Debug, PartialEq, Eq)]
#[cfg_attr(feature = "serde", derive(Serialize, Deserialize))]
pub struct PostRoomsStartRequestBody {
    /// Optional breakout room ID
    pub breakout_room: Option<BreakoutRoomId>,

    /// The resumption token for the room
    pub resumption: Option<ResumptionToken>,
}
