// SPDX-FileCopyrightText: OpenTalk GmbH <mail@opentalk.eu>
//
// SPDX-License-Identifier: EUPL-1.2

#[allow(unused_imports)]
use crate::imports::*;

use crate::{signaling::Role, sql_enum};

sql_enum!(
    feature_gated:

    #[derive(PartialEq, Eq)]
    #[cfg_attr(feature="serde", derive(Serialize, Deserialize), serde(rename_all = "snake_case"))]
    InviteRole,
    "invite_role",
    InviteRoleType,
    {
        User = b"user",
        Moderator = b"moderator",
    }
);

impl Default for InviteRole {
    fn default() -> Self {
        Self::User
    }
}

impl From<InviteRole> for Role {
    fn from(value: InviteRole) -> Self {
        match value {
            InviteRole::User => Role::User,
            InviteRole::Moderator => Role::Moderator,
        }
    }
}
