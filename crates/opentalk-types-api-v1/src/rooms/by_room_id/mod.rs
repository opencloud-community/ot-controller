// SPDX-FileCopyrightText: OpenTalk GmbH <mail@opentalk.eu>
//
// SPDX-License-Identifier: EUPL-1.2

//! Types related to the API endpoints under `/rooms/{room_id}`.

pub mod assets;
pub mod invites;

mod delete_room_query;
mod get_room_event_response_body;

pub use delete_room_query::DeleteRoomQuery;
pub use get_room_event_response_body::GetRoomEventResponseBody;
