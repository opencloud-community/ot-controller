// SPDX-FileCopyrightText: OpenTalk GmbH <mail@opentalk.eu>
//
// SPDX-License-Identifier: EUPL-1.2

use opentalk_types_common::{users::UserId, utils::ExampleData};

/// Public user details.
///
/// Contains general "public" information about a user. Is accessible to all other users.
#[derive(Clone, Debug, PartialEq, Eq)]
#[cfg_attr(feature = "serde", derive(serde::Serialize, serde::Deserialize))]
#[cfg_attr(feature = "utoipa", derive(utoipa::ToSchema), schema(
    example = json!(
        PublicUserProfile::example_data()
    )
))]
pub struct PublicUserProfile {
    /// The user id
    pub id: UserId,

    /// The email of the user
    pub email: String,

    /// The title of the user
    pub title: String,

    /// The user's first name
    pub firstname: String,

    /// The user's last name
    pub lastname: String,

    /// The user's display name
    pub display_name: String,

    /// The user's avatar URL
    pub avatar_url: String,
}

impl ExampleData for PublicUserProfile {
    fn example_data() -> Self {
        Self {
            id: UserId::from_u128(0xa11c3),
            email: "alice@example.com".to_string(),
            title: "".to_string(),
            firstname: "Alice".to_string(),
            lastname: "Adams".to_string(),
            display_name: "Alice Adams".to_string(),
            avatar_url: "https://gravatar.com/avatar/c160f8cc69a4f0bf2b0362752353d060".to_string(),
        }
    }
}
