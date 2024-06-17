// SPDX-FileCopyrightText: OpenTalk GmbH <mail@opentalk.eu>
//
// SPDX-License-Identifier: EUPL-1.2

use chrono::{DateTime, TimeZone, Utc};

#[allow(unused_imports)]
use crate::imports::*;
use crate::{
    api::v1::users::PublicUserProfile,
    core::{InviteCodeId, RoomId},
    utils::ExampleData,
};

/// Public invite details
///
/// Contains general public information about a room.
#[derive(Clone, Debug, PartialEq, Eq)]
#[cfg_attr(feature = "serde", derive(Serialize, Deserialize))]
#[cfg_attr(
    feature="utoipa",
    derive(utoipa::ToSchema),
    schema(example = json!(InviteResource::example_data())),
)]
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

impl ExampleData for InviteResource {
    fn example_data() -> Self {
        Self {
            invite_code: InviteCodeId::example_data(),
            created: Utc.with_ymd_and_hms(2024, 6, 18, 11, 22, 33).unwrap(),
            created_by: PublicUserProfile::example_data(),
            updated: Utc.with_ymd_and_hms(2024, 6, 20, 14, 16, 19).unwrap(),
            updated_by: PublicUserProfile::example_data(),
            room_id: RoomId::example_data(),
            active: true,
            expiration: None,
        }
    }
}
