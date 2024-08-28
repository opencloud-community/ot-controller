// SPDX-FileCopyrightText: OpenTalk GmbH <mail@opentalk.eu>
//
// SPDX-License-Identifier: EUPL-1.2

#[allow(unused_imports)]
use crate::imports::*;
use crate::{common::email::EmailAddress, core::EmailInviteRole, utils::ExampleData};

/// Request body variant for the `POST /events/{event_id}/invites` endpoint
#[derive(Debug, Eq, PartialEq, Clone)]
#[cfg_attr(feature = "serde", derive(Serialize, Deserialize))]
#[cfg_attr(feature = "utoipa", derive(utoipa::ToSchema), schema(
    example = json!(
        EmailInvite::example_data()
    )
))]
pub struct EmailInvite {
    /// Email address of the user to invite
    pub email: EmailAddress,
    #[cfg_attr(feature = "serde", serde(default))]
    /// Invite role of the user
    pub role: EmailInviteRole,
}

impl ExampleData for EmailInvite {
    fn example_data() -> Self {
        Self {
            email: EmailAddress::example_data(),
            role: EmailInviteRole::Guest,
        }
    }
}
