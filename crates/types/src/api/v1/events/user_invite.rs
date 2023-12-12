// SPDX-FileCopyrightText: OpenTalk GmbH <mail@opentalk.eu>
//
// SPDX-License-Identifier: EUPL-1.2

use crate::core::{InviteRole, UserId};
#[allow(unused_imports)]
use crate::imports::*;

/// Request body variant for the `POST /events/{event_id}/invites` endpoint
#[derive(Debug, Eq, PartialEq, Clone)]
#[cfg_attr(feature = "serde", derive(Serialize, Deserialize))]
pub struct UserInvite {
    /// ID of the user to invite
    pub invitee: UserId,
    #[cfg_attr(feature = "serde", serde(default))]
    /// Invite role of the user
    pub role: InviteRole,
}
