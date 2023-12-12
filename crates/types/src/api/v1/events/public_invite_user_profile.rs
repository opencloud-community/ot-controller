// SPDX-FileCopyrightText: OpenTalk GmbH <mail@opentalk.eu>
//
// SPDX-License-Identifier: EUPL-1.2

#[allow(unused_imports)]
use crate::imports::*;
use crate::{api::v1::users::PublicUserProfile, core::InviteRole};

/// Profile of a public event invitee
#[derive(Debug, Clone, PartialEq, Eq)]
#[cfg_attr(feature = "serde", derive(Serialize, Deserialize))]
pub struct PublicInviteUserProfile {
    #[cfg_attr(feature = "serde", serde(flatten))]
    /// Public user profile
    pub user_profile: PublicUserProfile,
    /// Invite role of the invitee
    pub role: InviteRole,
}
