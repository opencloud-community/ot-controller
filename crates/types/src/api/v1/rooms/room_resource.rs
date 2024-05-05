// SPDX-FileCopyrightText: OpenTalk GmbH <mail@opentalk.eu>
//
// SPDX-License-Identifier: EUPL-1.2

#[allow(unused_imports)]
use crate::imports::*;
use crate::{
    api::v1::users::PublicUserProfile,
    core::{RoomId, Timestamp},
};

/// A Room
///
/// Contains all room information. Is only be accessible to the owner and users with
/// appropriate permissions.
#[derive(Clone, Debug, PartialEq, Eq)]
#[cfg_attr(feature = "serde", derive(Serialize, Deserialize))]
#[cfg_attr(feature = "utoipa", derive(utoipa::ToSchema))]
pub struct RoomResource {
    /// The ID of the room
    pub id: RoomId,

    /// The public user profile of the room's owner
    pub created_by: PublicUserProfile,

    /// The date when the room was created
    pub created_at: Timestamp,

    /// The password of the room, if any
    pub password: Option<String>,

    /// If waiting room is enabled
    pub waiting_room: bool,
}
