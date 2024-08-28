// SPDX-FileCopyrightText: OpenTalk GmbH <mail@opentalk.eu>
//
// SPDX-License-Identifier: EUPL-1.2

#[allow(unused_imports)]
use crate::imports::*;
use crate::{events::invites::EmailInviteRole, sql_enum, utils::ExampleData};

sql_enum!(
    feature_gated:

    #[derive(PartialEq, Eq)]
    #[cfg_attr(feature="serde", derive(Serialize, Deserialize), serde(rename_all = "snake_case"))]
    #[cfg_attr(
        feature = "utoipa",
        derive(utoipa::ToSchema),
        schema(example = json!(InviteRole::example_data()))
    )]
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

impl From<EmailInviteRole> for InviteRole {
    fn from(value: EmailInviteRole) -> Self {
        match value {
            EmailInviteRole::Guest => Self::User,
            EmailInviteRole::Moderator => Self::Moderator,
        }
    }
}

impl ExampleData for InviteRole {
    fn example_data() -> Self {
        Self::User
    }
}
