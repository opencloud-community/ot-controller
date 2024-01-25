// SPDX-FileCopyrightText: OpenTalk GmbH <mail@opentalk.eu>
//
// SPDX-License-Identifier: EUPL-1.2

//! This module contains types that are used for OpenTalk API V1 rooms endpoints.

use strum::AsRefStr;

use crate::{
    common::event::EventInfo,
    core::{BreakoutRoomId, ResumptionToken, RoomId, TicketToken},
};

#[allow(unused_imports)]
use crate::imports::*;

mod post_rooms_request_body;
mod room_resource;

pub mod sip_config_resource;
pub mod streaming_targets;

pub use post_rooms_request_body::PostRoomsRequestBody;
pub use room_resource::RoomResource;

/// API request parameters to patch a room
#[derive(Clone, Debug)]
#[cfg_attr(feature = "serde", derive(Serialize, Deserialize, Validate))]
pub struct PatchRoomsRequestBody {
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

/// The JSON body expected when making a *POST /rooms/{room_id}/start_invited*
#[derive(Clone, Debug)]
#[cfg_attr(feature = "serde", derive(Serialize, Deserialize))]
pub struct InvitedStartRequest {
    /// The invited user's password to the room
    pub password: Option<String>,

    /// The invite code
    pub invite_code: String,

    /// Optional breakout room ID
    pub breakout_room: Option<BreakoutRoomId>,

    /// The resumption token for the room
    pub resumption: Option<ResumptionToken>,
}

/// A GET request to the `/rooms/<room_id>/event` endpoint
#[derive(Clone, Debug, PartialEq, Eq)]
#[cfg_attr(
    feature = "frontend",
    derive(HttpRequest),
    http_request(
        method = "GET",
        response = GetRoomEventResponse,
        path = "/v1/rooms/{0}/event"
    )
)]
pub struct GetRoomEventRequest(pub RoomId);

/// The JSON body returned by the `/rooms/<room_id>/event` endpoint
#[derive(Clone, Debug, PartialEq, Eq)]
#[cfg_attr(feature = "serde", derive(Serialize, Deserialize))]
pub struct GetRoomEventResponse(pub EventInfo);

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
