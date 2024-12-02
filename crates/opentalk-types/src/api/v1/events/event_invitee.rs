// SPDX-FileCopyrightText: OpenTalk GmbH <mail@opentalk.eu>
//
// SPDX-License-Identifier: EUPL-1.2

use opentalk_types_api_v1::events::EventInviteeProfile;
use opentalk_types_common::{events::invites::EventInviteStatus, utils::ExampleData};

#[allow(unused_imports)]
use crate::imports::*;

/// Invitee to an event
///
///  Contains user profile and invitee status
#[derive(Debug, Clone, PartialEq, Eq)]
#[cfg_attr(feature = "serde", derive(Serialize, Deserialize))]
#[cfg_attr(
    feature = "utoipa",
    derive(utoipa::ToSchema),
    schema(example = json!(EventInvitee::example_data()))
)]
pub struct EventInvitee {
    /// User profile of the invitee
    pub profile: EventInviteeProfile,
    /// Invite status of the invitee
    pub status: EventInviteStatus,
}

impl ExampleData for EventInvitee {
    fn example_data() -> Self {
        Self {
            profile: EventInviteeProfile::example_data(),
            status: EventInviteStatus::example_data(),
        }
    }
}
