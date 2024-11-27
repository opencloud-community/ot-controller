// SPDX-FileCopyrightText: OpenTalk GmbH <mail@opentalk.eu>
//
// SPDX-License-Identifier: EUPL-1.2

//! This module contains types that are used in OpenTalk API V1 users endpoints.

use opentalk_types_common::{users::UserTitle, utils::ExampleData};

#[allow(unused_imports)]
use crate::imports::*;

/// Used to modify user settings.
#[derive(Default, Clone, Debug, PartialEq, Eq)]
#[cfg_attr(feature = "serde", derive(Serialize, Deserialize, Validate))]
#[cfg_attr(feature = "utoipa", derive(utoipa::ToSchema), schema(
    example = json!(
        PatchMeBody::example_data()
    )
))]
pub struct PatchMeBody {
    /// The user's title
    // Field is non-required already, utoipa adds a `nullable: true` entry
    // by default which creates a false positive in the spectral linter when
    // combined with example data.
    #[cfg_attr(
        feature = "serde",
        serde(default, skip_serializing_if = "Option::is_none")
    )]
    #[cfg_attr(feature = "utoipa", schema(nullable = false))]
    pub title: Option<UserTitle>,

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

impl ExampleData for PatchMeBody {
    fn example_data() -> Self {
        Self {
            display_name: Some("Alice Adams".to_string()),
            language: Some("en".to_string()),
            ..Default::default()
        }
    }
}
