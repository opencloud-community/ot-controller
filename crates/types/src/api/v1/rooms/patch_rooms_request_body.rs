// SPDX-FileCopyrightText: OpenTalk GmbH <mail@opentalk.eu>
//
// SPDX-License-Identifier: EUPL-1.2

#[allow(unused_imports)]
use crate::imports::*;

/// API request parameters to patch a room
#[derive(Clone, Debug)]
#[cfg_attr(feature = "serde", derive(Serialize, Deserialize, Validate))]
pub struct PatchRoomsRequestBody {
    /// The password for the room
    #[cfg_attr(
        feature = "serde",
        validate(length(min = 1, max = 255)),
        serde(default, deserialize_with = "crate::api::v1::utils::deserialize_some")
    )]
    pub password: Option<Option<String>>,

    /// If waiting room is enabled
    pub waiting_room: Option<bool>,
}
