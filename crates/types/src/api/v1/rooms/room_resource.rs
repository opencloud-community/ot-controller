// SPDX-FileCopyrightText: OpenTalk GmbH <mail@opentalk.eu>
//
// SPDX-License-Identifier: EUPL-1.2

#[allow(unused_imports)]
use crate::imports::*;
use crate::{
    api::v1::users::PublicUserProfile,
    core::{RoomId, RoomPassword, Timestamp},
    utils::ExampleData,
};

/// A Room
///
/// Contains all room information. Is only be accessible to the owner and users with
/// appropriate permissions.
#[derive(Clone, Debug, PartialEq, Eq)]
#[cfg_attr(feature = "serde", derive(Serialize, Deserialize))]
#[cfg_attr(
    feature = "utoipa",
    derive(utoipa::ToSchema),
    schema(example = json!(
        RoomResource::example_data()
    ))
)]
pub struct RoomResource {
    /// The ID of the room
    pub id: RoomId,

    /// The public user profile of the room's owner
    pub created_by: PublicUserProfile,

    /// The date when the room was created
    pub created_at: Timestamp,

    /// The password of the room, if any
    #[cfg_attr(
        feature = "serde",
        serde(default, skip_serializing_if = "Option::is_none")
    )]
    // Field is non-required already, utoipa adds a `nullable: true` entry
    // by default which creates a false positive in the spectral linter when
    // combined with example data.
    #[cfg_attr(feature = "utoipa", schema(nullable = false))]
    pub password: Option<RoomPassword>,

    /// If waiting room is enabled
    pub waiting_room: bool,
}

impl ExampleData for RoomResource {
    fn example_data() -> Self {
        Self {
            id: RoomId::nil(),
            created_by: PublicUserProfile::example_data(),
            created_at: Timestamp::unix_epoch(),
            password: Some(RoomPassword::example_data()),
            waiting_room: false,
        }
    }
}
