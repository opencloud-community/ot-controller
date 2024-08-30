// SPDX-FileCopyrightText: OpenTalk GmbH <mail@opentalk.eu>
//
// SPDX-License-Identifier: EUPL-1.2

use opentalk_types_common::{rooms::RoomId, utils::ExampleData};

use super::CallInInfo;
use crate::core::RoomPassword;
#[allow(unused_imports)]
use crate::imports::*;

/// All information about a room in which an event takes place
#[derive(Debug, Clone, PartialEq, Eq)]
#[cfg_attr(feature = "serde", derive(Serialize, Deserialize))]
#[cfg_attr(
    feature = "utoipa",
    derive(utoipa::ToSchema),
    schema(example = json!(EventRoomInfo::example_data()))
)]
pub struct EventRoomInfo {
    /// ID of the room
    pub id: RoomId,

    /// Password of the room
    #[cfg_attr(
        feature = "serde",
        serde(default, skip_serializing_if = "Option::is_none")
    )]
    // Field is non-required already, utoipa adds a `nullable: true` entry
    // by default which creates a false positive in the spectral linter when
    // combined with example data.
    #[cfg_attr(feature = "utoipa", schema(nullable = false))]
    pub password: Option<RoomPassword>,

    /// Flag to check if the room has a waiting room enabled
    pub waiting_room: bool,

    /// Flag to check if the room has e2e encryption enabled
    pub e2e_encrytion: bool,

    /// Call-In information
    #[cfg_attr(
        feature = "serde",
        serde(default, skip_serializing_if = "Option::is_none")
    )]
    // Field is non-required already, utoipa adds a `nullable: true` entry
    // by default which creates a false positive in the spectral linter when
    // combined with example data.
    #[cfg_attr(feature = "utoipa", schema(nullable = false))]
    pub call_in: Option<CallInInfo>,
}

impl ExampleData for EventRoomInfo {
    fn example_data() -> Self {
        Self {
            id: RoomId::example_data(),
            password: Some(RoomPassword::example_data()),
            waiting_room: false,
            e2e_encrytion: false,
            call_in: Some(CallInInfo::example_data()),
        }
    }
}
