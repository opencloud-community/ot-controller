// SPDX-FileCopyrightText: OpenTalk GmbH <mail@opentalk.eu>
//
// SPDX-License-Identifier: EUPL-1.2

use email_address::EmailAddress;

#[allow(unused_imports)]
use crate::imports::*;

use super::UserInvite;

/// Request body for the `POST /events/{event_id}/invites` endpoint
#[derive(Debug, Eq, PartialEq, Clone)]
#[cfg_attr(feature = "serde", derive(Serialize, Deserialize), serde(untagged))]
pub enum PostEventInviteBody {
    /// Invite a registered user
    User(UserInvite),
    /// Invite a user by email
    Email {
        /// Email address of the user to invite
        email: EmailAddress,
    },
}
