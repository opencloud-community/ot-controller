// SPDX-FileCopyrightText: OpenTalk GmbH <mail@opentalk.eu>
//
// SPDX-License-Identifier: EUPL-1.2

use strum::AsRefStr;

#[allow(unused_imports)]
use crate::imports::*;

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
