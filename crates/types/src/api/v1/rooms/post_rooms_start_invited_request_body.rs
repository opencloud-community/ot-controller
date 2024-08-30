// SPDX-FileCopyrightText: OpenTalk GmbH <mail@opentalk.eu>
//
// SPDX-License-Identifier: EUPL-1.2

use opentalk_types_common::{
    auth::ResumptionToken,
    rooms::{invite_codes::InviteCode, BreakoutRoomId},
    utils::ExampleData,
};

use crate::core::RoomPassword;
#[allow(unused_imports)]
use crate::imports::*;

/// The JSON body expected when making a *POST /rooms/{room_id}/start_invited*
#[derive(Clone, Debug, PartialEq, Eq)]
#[cfg_attr(feature = "serde", derive(Serialize, Deserialize))]
#[cfg_attr(
    feature = "utoipa",
    derive(utoipa::ToSchema),
    schema(example = json!(PostRoomsStartInvitedRequestBody::example_data()))
)]
pub struct PostRoomsStartInvitedRequestBody {
    /// The invited user's password to the room
    #[cfg_attr(
        feature = "serde",
        serde(default, skip_serializing_if = "Option::is_none")
    )]
    // Field is non-required already, utoipa adds a `nullable: true` entry
    // by default which creates a false positive in the spectral linter when
    // combined with example data.
    #[cfg_attr(feature = "utoipa", schema(nullable = false))]
    pub password: Option<RoomPassword>,

    /// The invite code
    pub invite_code: String,

    /// Optional breakout room ID
    #[cfg_attr(
        feature = "serde",
        serde(default, skip_serializing_if = "Option::is_none")
    )]
    // Field is non-required already, utoipa adds a `nullable: true` entry
    // by default which creates a false positive in the spectral linter when
    // combined with example data.
    #[cfg_attr(feature = "utoipa", schema(nullable = false))]
    pub breakout_room: Option<BreakoutRoomId>,

    /// The resumption token for the room
    #[cfg_attr(
        feature = "serde",
        serde(default, skip_serializing_if = "Option::is_none")
    )]
    // Field is non-required already, utoipa adds a `nullable: true` entry
    // by default which creates a false positive in the spectral linter when
    // combined with example data.
    #[cfg_attr(feature = "utoipa", schema(nullable = false))]
    pub resumption: Option<ResumptionToken>,
}

impl ExampleData for PostRoomsStartInvitedRequestBody {
    fn example_data() -> Self {
        Self {
            password: Some(RoomPassword::example_data()),
            invite_code: InviteCode::example_data().to_string(),
            breakout_room: Some(BreakoutRoomId::example_data()),
            resumption: Some(ResumptionToken::example_data()),
        }
    }
}
