// SPDX-FileCopyrightText: OpenTalk GmbH <mail@opentalk.eu>
//
// SPDX-License-Identifier: EUPL-1.2

use crate::core::InviteRole;
#[allow(unused_imports)]
use crate::imports::*;

/// Request body for the `PATCH /events/{event_id}/invites/{user_id}` endpoint
#[derive(Debug, Clone, PartialEq, Eq)]
#[cfg_attr(feature = "serde", derive(Serialize, Deserialize))]
pub struct PatchInviteBody {
    /// Invite role of the user
    pub role: InviteRole,
}
