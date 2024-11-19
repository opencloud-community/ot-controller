// SPDX-FileCopyrightText: OpenTalk GmbH <mail@opentalk.eu>
//
// SPDX-License-Identifier: EUPL-1.2

//! This module contains types that are used for OpenTalk API V1 invites endpoints.

use opentalk_types_common::rooms::RoomId;

#[allow(unused_imports)]
use crate::imports::*;

mod post_invite_verify_request;
mod post_invite_verify_request_body;
mod put_invite_request_body;
mod room_id_and_invite_code;

pub use post_invite_verify_request::PostInviteVerifyRequest;
pub use post_invite_verify_request_body::PostInviteVerifyRequestBody;
pub use put_invite_request_body::PutInviteRequestBody;
pub use room_id_and_invite_code::RoomIdAndInviteCode;

/// Verify response body for *POST /invite/verify*
#[derive(Clone, Debug)]
#[cfg_attr(feature = "serde", derive(Serialize, Deserialize))]
#[cfg_attr(feature = "utoipa", derive(utoipa::ToSchema))]
pub struct CodeVerified {
    /// The room id for the invite
    pub room_id: RoomId,

    /// If password is required
    pub password_required: bool,
}
