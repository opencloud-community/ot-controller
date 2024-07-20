// SPDX-FileCopyrightText: OpenTalk GmbH <mail@opentalk.eu>
//
// SPDX-License-Identifier: EUPL-1.2

#[allow(unused_imports)]
use crate::imports::*;
use crate::{api::v1::users::PublicUserProfile, core::InviteRole, utils::ExampleData};

/// Profile of a public event invitee
#[derive(Debug, Clone, PartialEq, Eq)]
#[cfg_attr(feature = "serde", derive(Serialize, Deserialize))]
#[cfg_attr(
    feature = "utoipa",
    derive(utoipa::ToSchema),
    schema(example = json!(PublicInviteUserProfile::example_data()))
)]
pub struct PublicInviteUserProfile {
    #[cfg_attr(feature = "serde", serde(flatten))]
    /// Public user profile
    pub user_profile: PublicUserProfile,
    /// Invite role of the invitee
    pub role: InviteRole,
}

impl ExampleData for PublicInviteUserProfile {
    fn example_data() -> Self {
        Self {
            user_profile: PublicUserProfile::example_data(),
            role: InviteRole::example_data(),
        }
    }
}
