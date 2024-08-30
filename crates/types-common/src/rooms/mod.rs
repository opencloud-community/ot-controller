// SPDX-FileCopyrightText: OpenTalk GmbH <mail@opentalk.eu>
//
// SPDX-License-Identifier: EUPL-1.2

//! Data types for handling rooms.

pub mod invite_codes;

mod breakout_room_id;
mod room_id;

pub use breakout_room_id::BreakoutRoomId;
pub use room_id::RoomId;
