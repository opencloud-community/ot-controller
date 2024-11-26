// SPDX-FileCopyrightText: OpenTalk GmbH <mail@opentalk.eu>
//
// SPDX-License-Identifier: EUPL-1.2

//! Types related to the API endpoints under `/rooms`.

pub mod by_room_id;
pub mod streaming_targets;

mod room_resource;

pub use room_resource::RoomResource;
