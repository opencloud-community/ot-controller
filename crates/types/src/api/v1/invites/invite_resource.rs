// SPDX-FileCopyrightText: OpenTalk GmbH <mail@opentalk.eu>
//
// SPDX-License-Identifier: EUPL-1.2

use chrono::{DateTime, Utc};

#[allow(unused_imports)]
use crate::imports::*;
use crate::{
    api::v1::users::PublicUserProfile,
    core::{InviteCodeId, RoomId},
};

/// Public invite details
///
/// Contains general public information about a room.
#[derive(Clone, Debug, PartialEq, Eq)]
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
