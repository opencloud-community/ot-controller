// SPDX-FileCopyrightText: OpenTalk GmbH <mail@opentalk.eu>
//
// SPDX-License-Identifier: EUPL-1.2

//! This module contains types that are used for OpenTalk API V1 rooms endpoints.

use chrono::{DateTime, Utc};
use strum::AsRefStr;

use crate::core::{BreakoutRoomId, ResumptionToken, RoomId, TicketToken};

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

/// The JSON body expected when making a *POST /rooms/{room_id}/start*
#[derive(Clone, Debug)]
#[cfg_attr(feature = "serde", derive(Serialize, Deserialize))]
pub struct StartRequest {
    /// Optional breakout room ID
    pub breakout_room: Option<BreakoutRoomId>,

    /// The resumption token for the room
    pub resumption: Option<ResumptionToken>,
}

/// The JSON body returned from the start endpoints supporting session resumption
#[derive(Clone, Debug)]
#[cfg_attr(feature = "serde", derive(Serialize, Deserialize))]
pub struct StartResponse {
    /// The ticket token for the room
    pub ticket: TicketToken,

    /// The resumption token for the room
    pub resumption: ResumptionToken,
}

/// Errors for the /rooms/{room_id}/start* endpoint
#[derive(Clone, Debug, AsRefStr)]
#[strum(serialize_all = "snake_case")]
pub enum StartRoomError {
    /// The provided room password is wrong
    WrongRoomPassword,

    /// The requested room has no breakout rooms enabled
    NoBreakoutRooms,

    /// The provided breakout room ID is invalid
    InvalidBreakoutRoomId,

    /// The user requesting to start the room is banned from the room
    BannedFromRoom,
}

#[cfg(test)]
mod tests {
    use super::*;
    use pretty_assertions::assert_eq;

    #[test]
    fn to_string() {
        assert_eq!(
            StartRoomError::WrongRoomPassword.as_ref(),
            "wrong_room_password"
        );
        assert_eq!(
            StartRoomError::NoBreakoutRooms.as_ref(),
            "no_breakout_rooms"
        );
        assert_eq!(
            StartRoomError::InvalidBreakoutRoomId.as_ref(),
            "invalid_breakout_room_id"
        );
        assert_eq!(StartRoomError::BannedFromRoom.as_ref(), "banned_from_room");
    }
}
