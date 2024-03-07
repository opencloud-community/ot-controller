// SPDX-FileCopyrightText: OpenTalk GmbH <mail@opentalk.eu>
//
// SPDX-License-Identifier: EUPL-1.2

#[allow(unused_imports)]
use crate::imports::*;

/// API request parameters to create a new room
#[derive(Clone, Debug)]
#[cfg_attr(feature = "serde", derive(Serialize, Deserialize, Validate))]
pub struct PostRoomsRequestBody {
    /// The password to the room, if any
    #[cfg_attr(feature = "serde", validate(length(min = 1, max = 255)))]
    pub password: Option<String>,

    /// Enable/Disable sip for this room; defaults to false when not set
    #[cfg_attr(feature = "serde", serde(default))]
    pub enable_sip: bool,

    /// If waiting room is enabled
    #[cfg_attr(feature = "serde", serde(default))]
    pub waiting_room: bool,
}
