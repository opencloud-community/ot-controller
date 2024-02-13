// SPDX-FileCopyrightText: OpenTalk GmbH <mail@opentalk.eu>
//
// SPDX-License-Identifier: EUPL-1.2

use super::{EmailInvite, UserInvite};
#[allow(unused_imports)]
use crate::imports::*;

/// Request body for the `POST /events/{event_id}/invites` endpoint
#[derive(Debug, Eq, PartialEq, Clone)]
#[cfg_attr(feature = "serde", derive(Serialize, Deserialize), serde(untagged))]
pub enum PostEventInviteBody {
    /// Invite a registered user
    User(UserInvite),
    /// Invite a user by email
    Email(EmailInvite),
}
