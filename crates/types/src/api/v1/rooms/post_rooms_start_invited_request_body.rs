// SPDX-FileCopyrightText: OpenTalk GmbH <mail@opentalk.eu>
//
// SPDX-License-Identifier: EUPL-1.2

use crate::core::{BreakoutRoomId, ResumptionToken, RoomPassword};
#[allow(unused_imports)]
use crate::imports::*;

/// The JSON body expected when making a *POST /rooms/{room_id}/start_invited*
#[derive(Clone, Debug, PartialEq, Eq)]
#[cfg_attr(feature = "serde", derive(Serialize, Deserialize))]
#[cfg_attr(feature = "utoipa", derive(utoipa::ToSchema))]
pub struct PostRoomsStartInvitedRequestBody {
    /// The invited user's password to the room
    pub password: Option<RoomPassword>,

    /// The invite code
    pub invite_code: String,

    /// Optional breakout room ID
    pub breakout_room: Option<BreakoutRoomId>,

    /// The resumption token for the room
    pub resumption: Option<ResumptionToken>,
}
