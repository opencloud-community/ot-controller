// SPDX-FileCopyrightText: OpenTalk GmbH <mail@opentalk.eu>
//
// SPDX-License-Identifier: EUPL-1.2

use opentalk_types_api_v1::events::EmailInvite;
use opentalk_types_common::utils::ExampleData;

use super::UserInvite;
#[allow(unused_imports)]
use crate::imports::*;

/// Request body for the `POST /events/{event_id}/invites` endpoint
#[derive(Debug, Eq, PartialEq, Clone)]
#[cfg_attr(feature = "serde", derive(Serialize, Deserialize), serde(untagged))]
#[cfg_attr(feature = "utoipa", derive(utoipa::ToSchema), schema(
    example = json!(
        PostEventInviteBody::example_data()
    )
))]
pub enum PostEventInviteBody {
    /// Invite a registered user
    User(UserInvite),
    /// Invite a user by email
    Email(EmailInvite),
}

impl ExampleData for PostEventInviteBody {
    fn example_data() -> Self {
        Self::User(UserInvite::example_data())
    }
}
