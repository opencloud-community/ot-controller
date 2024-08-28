// SPDX-FileCopyrightText: OpenTalk GmbH <mail@opentalk.eu>
//
// SPDX-License-Identifier: EUPL-1.2

#[allow(unused_imports)]
use crate::imports::*;
use crate::{api::v1::events::EventStatus, core::DateTimeTz, utils::ExampleData};

/// Request body for the `PATCH /events/{event_id}/{instance_id}` endpoint
#[derive(Debug, Clone, PartialEq, Eq)]
#[cfg_attr(feature = "serde", derive(Serialize, Deserialize, Validate))]
#[cfg_attr(feature = "utoipa", derive(utoipa::ToSchema), schema(
    example = json!(
        PatchEventInstanceBody::example_data()
    )
))]
pub struct PatchEventInstanceBody {
    /// The title of th event
    #[cfg_attr(
        feature = "serde",
        serde(default, skip_serializing_if = "Option::is_none"),
        validate(length(max = 255))
    )]
    // Field is non-required already, utoipa adds a `nullable: true` entry
    // by default which creates a false positive in the spectral linter when
    // combined with example data.
    #[cfg_attr(feature = "utoipa", schema(nullable = false))]
    pub title: Option<String>,

    /// The description of the event
    #[cfg_attr(
        feature = "serde",
        serde(default, skip_serializing_if = "Option::is_none"),
        validate(length(max = 4096))
    )]
    // Field is non-required already, utoipa adds a `nullable: true` entry
    // by default which creates a false positive in the spectral linter when
    // combined with example data.
    #[cfg_attr(feature = "utoipa", schema(nullable = false))]
    pub description: Option<String>,

    /// Flag to indicate if the event is all-day
    #[cfg_attr(
        feature = "serde",
        serde(default, skip_serializing_if = "Option::is_none")
    )]
    // Field is non-required already, utoipa adds a `nullable: true` entry
    // by default which creates a false positive in the spectral linter when
    // combined with example data.
    #[cfg_attr(feature = "utoipa", schema(nullable = false))]
    pub is_all_day: Option<bool>,

    /// Start time of the event.
    #[cfg_attr(
        feature = "serde",
        serde(default, skip_serializing_if = "Option::is_none")
    )]
    // Field is non-required already, utoipa adds a `nullable: true` entry
    // by default which creates a false positive in the spectral linter when
    // combined with example data.
    #[cfg_attr(feature = "utoipa", schema(nullable = false))]
    pub starts_at: Option<DateTimeTz>,

    /// End time of the event.
    #[cfg_attr(
        feature = "serde",
        serde(default, skip_serializing_if = "Option::is_none")
    )]
    // Field is non-required already, utoipa adds a `nullable: true` entry
    // by default which creates a false positive in the spectral linter when
    // combined with example data.
    #[cfg_attr(feature = "utoipa", schema(nullable = false))]
    pub ends_at: Option<DateTimeTz>,

    /// Status of the event
    #[cfg_attr(
        feature = "serde",
        serde(default, skip_serializing_if = "Option::is_none")
    )]
    // Field is non-required already, utoipa adds a `nullable: true` entry
    // by default which creates a false positive in the spectral linter when
    // combined with example data.
    #[cfg_attr(feature = "utoipa", schema(nullable = false))]
    pub status: Option<EventStatus>,
}

impl PatchEventInstanceBody {
    /// Check if the body is empty
    pub fn is_empty(&self) -> bool {
        let PatchEventInstanceBody {
            title,
            description,
            is_all_day,
            starts_at,
            ends_at,
            status,
        } = self;

        title.is_none()
            && description.is_none()
            && is_all_day.is_none()
            && starts_at.is_none()
            && ends_at.is_none()
            && status.is_none()
    }
}

impl ExampleData for PatchEventInstanceBody {
    fn example_data() -> Self {
        Self {
            title: Some("Early morning meeting".to_string()),
            description: None,
            is_all_day: Some(false),
            starts_at: None,
            ends_at: None,
            status: Some(EventStatus::Cancelled),
        }
    }
}
