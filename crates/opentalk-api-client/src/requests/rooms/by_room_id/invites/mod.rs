// SPDX-FileCopyrightText: OpenTalk GmbH <mail@opentalk.eu>
//
// SPDX-License-Identifier: EUPL-1.2

//! Requests for the API endpoints under `/rooms/{room_id}/invites`.

mod post_invite_verify_request;

pub use post_invite_verify_request::PostInviteVerifyRequest;
