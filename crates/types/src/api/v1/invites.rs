// SPDX-FileCopyrightText: OpenTalk GmbH <mail@opentalk.eu>
//
// SPDX-License-Identifier: EUPL-1.2

//! This module contains types that are used for OpenTalk API V1 invites endpoints.

use chrono::{DateTime, Utc};

use crate::core::{InviteCodeId, RoomId};
#[allow(unused_imports)]
use crate::imports::*;

use super::users::PublicUserProfile;

#[cfg(feature = "frontend")]
const VERIFY_PATH: &str = "/v1/invite/verify";

/// Public invite details
///
/// Contains general public information about a room.
#[derive(Clone, Debug)]
#[cfg_attr(feature = "serde", derive(Serialize, Deserialize))]
pub struct InviteResource {
    /// The invite code id
    pub invite_code: InviteCodeId,

    /// The timestamp this invite was created at
    pub created: DateTime<Utc>,

    /// The user who created the invite
    pub created_by: PublicUserProfile,

    /// The timestamp this invite was updated at
    pub updated: DateTime<Utc>,

    /// The user who updated the invite
    pub updated_by: PublicUserProfile,

    /// The room id for the invite
    pub room_id: RoomId,

    /// If the invite is active
    pub active: bool,

    /// Optional expiration date of the invite
    pub expiration: Option<DateTime<Utc>>,
}

/// Body for *POST /rooms/{room_id}/invites*
#[derive(Clone, Debug)]
#[cfg_attr(feature = "serde", derive(Serialize, Deserialize))]
pub struct PostInviteBody {
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
#[cfg_attr(feature = "serde", derive(Serialize, Deserialize, Validate))]
pub struct VerifyBody {
    /// The invite code id
    pub invite_code: InviteCodeId,
}

#[cfg(feature = "frontend")]
impl Request for VerifyBody {
    type Response = CodeVerified;
    const METHOD: Method = Method::POST;

    fn path(&self) -> String {
        VERIFY_PATH.into()
    }
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
