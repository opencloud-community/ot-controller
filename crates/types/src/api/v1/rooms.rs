// SPDX-FileCopyrightText: OpenTalk GmbH <mail@opentalk.eu>
//
// SPDX-License-Identifier: EUPL-1.2

//! This module contains types that are used for OpenTalk API V1 rooms endpoints.

use chrono::{DateTime, Utc};

use crate::core::RoomId;

#[allow(unused_imports)]
use crate::imports::*;

use super::users::PublicUserProfile;

/// A Room
///
/// Contains all room information. Is only be accessible to the owner and users with
/// appropriate permissions.
#[derive(Clone, Debug)]
#[cfg_attr(feature = "serde", derive(Serialize, Deserialize))]
pub struct RoomResource {
    /// The ID of the room
    pub id: RoomId,

    /// The public user profile of the room's owner
    pub created_by: PublicUserProfile,

    /// The date when the room was created
    pub created_at: DateTime<Utc>,

    /// The password of the room, if any
    pub password: Option<String>,

    /// If waiting room is enabled
    pub waiting_room: bool,
}

/// API request parameters to create a new room
#[derive(Clone, Debug)]
#[cfg_attr(feature = "serde", derive(Serialize, Deserialize, Validate))]
pub struct PostRoomsBody {
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

/// API request parameters to patch a room
#[derive(Clone, Debug)]
#[cfg_attr(feature = "serde", derive(Serialize, Deserialize, Validate))]
pub struct PatchRoomsBody {
    /// The password for the room
    #[cfg_attr(
        feature = "serde",
        validate(length(min = 1, max = 255)),
        serde(default, deserialize_with = "super::utils::deserialize_some")
    )]
    pub password: Option<Option<String>>,

    /// If waiting room is enabled
    pub waiting_room: Option<bool>,
}
