// SPDX-FileCopyrightText: OpenTalk GmbH <mail@opentalk.eu>
//
// SPDX-License-Identifier: EUPL-1.2

#[allow(unused_imports)]
use crate::imports::*;
use crate::{common::email::EmailAddress, core::EmailInviteRole};

/// Request body for the `PATCH /events/{event_id}/invites/email` endpoint
#[derive(Debug, Clone, PartialEq, Eq)]
#[cfg_attr(feature = "serde", derive(Serialize, Deserialize))]
pub struct PatchEmailInviteBody {
    /// Email address of the user to modify the invite for
    pub email: EmailAddress,
    /// Invite role of the user
    pub role: Option<EmailInviteRole>,
}
