// SPDX-FileCopyrightText: OpenTalk GmbH <mail@opentalk.eu>
//
// SPDX-License-Identifier: EUPL-1.2

use types::core::{BreakoutRoomId, RoomId};

/// The complete room id
///
/// It consist of the room-id inside the database and an optional
/// breakout-room-id which is generated when the breakout rooms are created
#[derive(Debug, Copy, Clone)]
pub struct SignalingRoomId(RoomId, Option<BreakoutRoomId>);

impl SignalingRoomId {
    pub const fn nil() -> Self {
        Self(RoomId::nil(), None)
    }

    pub const fn new(room: RoomId, breakout_room: Option<BreakoutRoomId>) -> Self {
        Self(room, breakout_room)
    }

    pub const fn new_for_room(room: RoomId) -> Self {
        Self(room, None)
    }

    pub const fn room_id(&self) -> RoomId {
        self.0
    }

    pub const fn breakout_room_id(&self) -> Option<BreakoutRoomId> {
        self.1
    }
}

impl std::fmt::Display for SignalingRoomId {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        if let Some(breakout) = self.1 {
            write!(f, "{}:{}", self.0, breakout)
        } else {
            self.0.fmt(f)
        }
    }
}
