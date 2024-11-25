// SPDX-FileCopyrightText: OpenTalk GmbH <mail@opentalk.eu>
//
// SPDX-License-Identifier: EUPL-1.2

//! Types related to the API endpoints under `/rooms/{room_id}`.

pub mod assets;
pub mod invites;

mod delete_room_query;

pub use delete_room_query::DeleteRoomQuery;
