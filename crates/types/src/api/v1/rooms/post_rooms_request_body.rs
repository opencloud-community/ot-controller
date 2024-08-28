// SPDX-FileCopyrightText: OpenTalk GmbH <mail@opentalk.eu>
//
// SPDX-License-Identifier: EUPL-1.2

use crate::core::RoomPassword;
#[allow(unused_imports)]
use crate::imports::*;

/// API request parameters to create a new room
#[derive(Clone, Debug)]
#[cfg_attr(feature = "serde", derive(Serialize, Deserialize))]
#[cfg_attr(feature = "utoipa", derive(utoipa::ToSchema))]
pub struct PostRoomsRequestBody {
    /// The password to the room, if any
    pub password: Option<RoomPassword>,

    /// Enable/Disable sip for this room; defaults to false when not set
    #[cfg_attr(feature = "serde", serde(default))]
    pub enable_sip: bool,

    /// Indicates whether the meeting room should have the waiting room enabled.
    /// When not present, the waiting room will be disabled.
    #[cfg_attr(feature = "serde", serde(default))]
    pub waiting_room: bool,
}
