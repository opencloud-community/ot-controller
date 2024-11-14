// SPDX-FileCopyrightText: OpenTalk GmbH <mail@opentalk.eu>
//
// SPDX-License-Identifier: EUPL-1.2

use opentalk_types_common::rooms::RoomPassword;

#[allow(unused_imports)]
use crate::imports::*;

/// API request parameters to patch a room
#[derive(Clone, Debug)]
#[cfg_attr(feature = "serde", derive(Serialize, Deserialize,))]
#[cfg_attr(feature = "utoipa", derive(utoipa::ToSchema))]
pub struct PatchRoomsRequestBody {
    /// The password for the room
    #[cfg_attr(
        feature = "serde",
        serde(default, deserialize_with = "crate::api::v1::utils::deserialize_some")
    )]
    pub password: Option<Option<RoomPassword>>,

    /// If waiting room is enabled
    pub waiting_room: Option<bool>,

    /// If e2e encryption is enabled
    pub e2e_encryption: Option<bool>,
}
