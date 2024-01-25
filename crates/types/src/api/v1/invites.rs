// SPDX-FileCopyrightText: OpenTalk GmbH <mail@opentalk.eu>
//
// SPDX-License-Identifier: EUPL-1.2

//! This module contains types that are used for OpenTalk API V1 invites endpoints.

use crate::core::{InviteCodeId, RoomId};
#[allow(unused_imports)]
use crate::imports::*;

mod invite_resource;
mod post_invite_request_body;
mod put_invite_request_body;
mod room_id_and_invite_code;

pub use invite_resource::InviteResource;
pub use post_invite_request_body::PostInviteRequestBody;
pub use put_invite_request_body::PutInviteRequestBody;
pub use room_id_and_invite_code::RoomIdAndInviteCode;

/// Verify body for *POST /invite/verify*
#[derive(Clone, Debug)]
#[cfg_attr(
    feature = "frontend",
    derive(HttpRequest),
    http_request(method = "POST", response = CodeVerified, path = "/v1/invite/verify")
)]
#[cfg_attr(feature = "serde", derive(Serialize, Deserialize, Validate))]
pub struct VerifyBody {
    /// The invite code id
    pub invite_code: InviteCodeId,
}

/// Verify response body for *POST /invite/verify*
#[derive(Clone, Debug)]
#[cfg_attr(feature = "serde", derive(Serialize, Deserialize))]
pub struct CodeVerified {
    /// The room id for the invite
    pub room_id: RoomId,

    /// If password is required
    pub password_required: bool,
}
