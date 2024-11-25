// SPDX-FileCopyrightText: OpenTalk GmbH <mail@opentalk.eu>
//
// SPDX-License-Identifier: EUPL-1.2

//! Requests for the API endpoints under `/rooms/{room_id}`.

pub mod invites;

mod get_room_event_request;

pub use get_room_event_request::GetRoomEventRequest;
