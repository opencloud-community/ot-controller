// SPDX-FileCopyrightText: OpenTalk GmbH <mail@opentalk.eu>
//
// SPDX-License-Identifier: EUPL-1.2

//! This module contains types that are used for OpenTalk API V1 invites endpoints.

use chrono::{DateTime, Utc};

use crate::core::{InviteCodeId, RoomId};
#[allow(unused_imports)]
use crate::imports::*;

mod invite_resource;

pub use invite_resource::InviteResource;

/// Body for *POST /rooms/{room_id}/invites*
#[derive(Clone, Debug, PartialEq, Eq)]
#[cfg_attr(feature = "serde", derive(Serialize, Deserialize))]
pub struct PostInviteRequestBody {
    /// Optional expiration date of the invite
    pub expiration: Option<DateTime<Utc>>,
}

/// Body for *GET /rooms/{room_id}/invites/{invite_code}*
#[derive(Clone, Debug)]
#[cfg_attr(feature = "serde", derive(Serialize, Deserialize))]
pub struct RoomIdAndInviteCode {
    /// The room id for the invite
    pub room_id: RoomId,

    /// The invite code id
    pub invite_code: InviteCodeId,
}

/// Body for *PUT /rooms/{room_id}/invites/{invite_code}*
#[derive(Clone, Debug)]
#[cfg_attr(feature = "serde", derive(Serialize, Deserialize))]
pub struct PutInviteBody {
    /// Optional expiration date of the invite
    pub expiration: Option<DateTime<Utc>>,
}

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
