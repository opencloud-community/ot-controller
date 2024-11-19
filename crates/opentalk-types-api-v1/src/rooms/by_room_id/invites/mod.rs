// SPDX-FileCopyrightText: OpenTalk GmbH <mail@opentalk.eu>
//
// SPDX-License-Identifier: EUPL-1.2

//! Types related to the API endpoints under `/rooms/{room_id}/invites`.

mod get_rooms_invites_response_body;
mod invite_resource;

pub use get_rooms_invites_response_body::GetRoomsInvitesResponseBody;
pub use invite_resource::InviteResource;
