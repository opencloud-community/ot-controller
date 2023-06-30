// SPDX-FileCopyrightText: OpenTalk GmbH <mail@opentalk.eu>
//
// SPDX-License-Identifier: EUPL-1.2

//! This module contains types that are used for OpenTalk API V1 rooms endpoints.

use chrono::{DateTime, Utc};

use crate::core::RoomId;

#[allow(unused_imports)]
use crate::imports::*;

use super::users::PublicUserProfile;

/// A Room
///
/// Contains all room information. Is only be accessible to the owner and users with
/// appropriate permissions.
#[derive(Clone, Debug)]
#[cfg_attr(feature = "serde", derive(Serialize, Deserialize))]
pub struct RoomResource {
    /// The ID of the room
    pub id: RoomId,

    /// The public user profile of the room's owner
    pub created_by: PublicUserProfile,

    /// The date when the room was created
    pub created_at: DateTime<Utc>,

    /// The password of the room, if any
    pub password: Option<String>,

    /// If waiting room is enabled
    pub waiting_room: bool,
}
