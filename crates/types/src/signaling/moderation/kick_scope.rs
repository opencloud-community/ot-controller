// SPDX-FileCopyrightText: OpenTalk GmbH <mail@opentalk.eu>
//
// SPDX-License-Identifier: EUPL-1.2

use crate::signaling::Role;

#[allow(unused_imports)]
use crate::imports::*;

/// The scope of users to be kicked from the room
#[derive(Clone, Copy, Debug, PartialEq, Eq)]
#[cfg_attr(
    feature = "serde",
    derive(Serialize, Deserialize),
    serde(tag = "kick_scope", rename_all = "snake_case")
)]
pub enum KickScope {
    /// Only kick guests from the room
    Guests,

    /// Kick both users and guests from the room but not moderators
    UsersAndGuests,

    /// Kick every participant from the room
    All,
}

impl KickScope {
    /// Query whether a specific role is kicked by the scope
    pub const fn kicks_role(&self, role: Role) -> bool {
        match self {
            KickScope::Guests => matches!(role, Role::Guest),
            KickScope::UsersAndGuests => !matches!(role, Role::Moderator),
            KickScope::All => true,
        }
    }
}
