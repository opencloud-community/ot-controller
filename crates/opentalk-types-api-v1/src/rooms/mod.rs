// SPDX-FileCopyrightText: OpenTalk GmbH <mail@opentalk.eu>
//
// SPDX-License-Identifier: EUPL-1.2

//! Types related to the API endpoints under `/rooms`.

pub mod by_room_id;
pub mod streaming_targets;

mod get_rooms_response_body;
mod room_resource;

pub use get_rooms_response_body::GetRoomsResponseBody;
pub use room_resource::RoomResource;
