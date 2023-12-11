// SPDX-FileCopyrightText: OpenTalk GmbH <mail@opentalk.eu>
//
// SPDX-License-Identifier: EUPL-1.2

use crate::api::v1::users::UnregisteredUser;
#[allow(unused_imports)]
use crate::imports::*;

use super::{EmailOnlyUser, PublicInviteUserProfile};

/// Profile of an event invitee
#[derive(Debug, Clone, PartialEq, Eq)]
#[cfg_attr(
    feature = "serde",
    derive(Serialize, Deserialize),
    serde(tag = "kind", rename_all = "lowercase")
)]
pub enum EventInviteeProfile {
    /// Registered user profile
    Registered(PublicInviteUserProfile),
    /// Unregistered user profile
    Unregistered(UnregisteredUser),
    /// Email only user profile
    Email(EmailOnlyUser),
}
