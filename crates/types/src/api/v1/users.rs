// SPDX-FileCopyrightText: OpenTalk GmbH <mail@opentalk.eu>
//
// SPDX-License-Identifier: EUPL-1.2

//! This module contains types that are used in OpenTalk API V1 users endpoints.

use crate::core::{TariffStatus, UserId};

#[allow(unused_imports)]
use crate::imports::*;

/// Public user details.
///
/// Contains general "public" information about a user. Is accessible to all other users.
#[derive(Clone, Debug, PartialEq, Eq)]
#[cfg_attr(feature = "serde", derive(Serialize, Deserialize))]
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

/// Private user profile.
///
/// Similar to [`PublicUserProfile`], but contains additional "private" information about a user.
/// Is only accessible to the user himself.
/// Is used on */users/me* endpoints.
#[derive(Clone, Debug)]
#[cfg_attr(feature = "serde", derive(Serialize, Deserialize))]
pub struct PrivateUserProfile {
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

    /// The dashboard theme
    pub dashboard_theme: String,

    /// The conference theme
    pub conference_theme: String,

    /// The language for the user
    pub language: String,

    /// The tariff status of the user
    pub tariff_status: TariffStatus,
}

/// Used to modify user settings.
#[derive(Clone, Debug)]
#[cfg_attr(feature = "serde", derive(Serialize, Deserialize, Validate))]
pub struct PatchMeBody {
    /// The user's title
    #[cfg_attr(feature = "serde", validate(length(max = 255)))]
    pub title: Option<String>,

    /// The user's display name
    #[cfg_attr(feature = "serde", validate(length(max = 255)))]
    pub display_name: Option<String>,

    /// The user's language
    #[cfg_attr(feature = "serde", validate(length(max = 35)))]
    pub language: Option<String>,

    /// The dashboard theme
    #[cfg_attr(feature = "serde", validate(length(max = 128)))]
    pub dashboard_theme: Option<String>,

    /// The conference theme
    #[cfg_attr(feature = "serde", validate(length(max = 128)))]
    pub conference_theme: Option<String>,
}

impl PatchMeBody {
    /// Check if any field is empty in `PatchMeBody`.
    pub fn is_empty(&self) -> bool {
        let PatchMeBody {
            title,
            display_name,
            language,
            dashboard_theme,
            conference_theme,
        } = self;

        title.is_none()
            && display_name.is_none()
            && language.is_none()
            && dashboard_theme.is_none()
            && conference_theme.is_none()
    }
}

/// The query string for finding a user
#[derive(Clone, Debug)]
#[cfg_attr(feature = "serde", derive(Serialize, Deserialize))]
pub struct GetFindQuery {
    /// The query string
    pub q: String,
}

/// The response for users found
#[derive(Clone, Debug)]
#[cfg_attr(
    feature = "serde",
    derive(Serialize, Deserialize),
    serde(tag = "kind", rename_all = "lowercase")
)]
pub enum GetFindResponseItem {
    /// Registered user
    Registered(PublicUserProfile),

    /// Unregistered user
    Unregistered(UnregisteredUser),
}

/// Representation of a unregistered user
#[derive(Clone, Debug, PartialEq, Eq)]
#[cfg_attr(feature = "serde", derive(Serialize, Deserialize))]
pub struct UnregisteredUser {
    /// Email of the unregistered user
    pub email: String,

    /// First name of the unregistered user
    pub firstname: String,

    /// Last name of the unregistered user
    pub lastname: String,

    /// Avatar URL for the unregistered user
    pub avatar_url: String,
}

/// Response body for the `GET /users/me/pending_invites` endpoint
#[derive(Clone, Debug, Eq, PartialEq)]
#[cfg_attr(feature = "serde", derive(Serialize, Deserialize))]
pub struct GetEventInvitesPendingResponse {
    /// Number of pending invites
    pub total_pending_invites: u32,
}
