// SPDX-FileCopyrightText: OpenTalk GmbH <mail@opentalk.eu>
//
// SPDX-License-Identifier: EUPL-1.2

use opentalk_types_common::utils::ExampleData;

use crate::core::{InviteRole, UserId};
#[allow(unused_imports)]
use crate::imports::*;

/// Request body variant for the `POST /events/{event_id}/invites` endpoint
#[derive(Debug, Eq, PartialEq, Clone)]
#[cfg_attr(feature = "serde", derive(Serialize, Deserialize))]
#[cfg_attr(feature = "utoipa", derive(utoipa::ToSchema), schema(
    example = json!(
        UserInvite::example_data()
    )
))]
pub struct UserInvite {
    /// ID of the user to invite
    pub invitee: UserId,
    #[cfg_attr(feature = "serde", serde(default))]
    /// Invite role of the user
    pub role: InviteRole,
}

impl ExampleData for UserInvite {
    fn example_data() -> Self {
        Self {
            invitee: UserId::example_data(),
            role: InviteRole::User,
        }
    }
}
