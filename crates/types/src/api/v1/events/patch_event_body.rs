// SPDX-FileCopyrightText: OpenTalk GmbH <mail@opentalk.eu>
//
// SPDX-License-Identifier: EUPL-1.2

use opentalk_types_common::{
    rooms::RoomPassword,
    streaming::StreamingTarget,
    time::{DateTimeTz, RecurrencePattern},
    utils::ExampleData,
};

#[cfg(feature = "serde")]
use crate::api::v1::utils::deserialize_some;
#[allow(unused_imports)]
use crate::imports::*;

/// Body for the `PATCH /events/{event_id}` endpoint
#[derive(Debug, Clone, PartialEq, Eq)]
#[cfg_attr(feature = "serde", derive(Serialize, Deserialize, Validate))]
#[cfg_attr(feature = "utoipa", derive(utoipa::ToSchema), schema(
    example = json!(
        PatchEventBody::example_data()
    )
))]
pub struct PatchEventBody {
    /// Patch the title of th event
    #[cfg_attr(
        feature = "serde",
        serde(default, skip_serializing_if = "Option::is_none"),
        validate(length(max = 255))
    )]
    pub title: Option<String>,

    /// Patch the description of the event
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

    /// Patch the password of the event's room
    #[cfg_attr(
        feature = "serde",
        serde(
            default,
            deserialize_with = "deserialize_some",
            skip_serializing_if = "Option::is_none"
        )
    )]
    // Field is non-required already, utoipa adds a `nullable: true` entry
    // by default which creates a false positive in the spectral linter when
    // combined with example data.
    #[cfg_attr(feature = "utoipa", schema(nullable = false))]
    pub password: Option<Option<RoomPassword>>,

    /// Patch the presence of a waiting room
    #[cfg_attr(
        feature = "serde",
        serde(default, skip_serializing_if = "Option::is_none")
    )]
    // Field is non-required already, utoipa adds a `nullable: true` entry
    // by default which creates a false positive in the spectral linter when
    // combined with example data.
    #[cfg_attr(feature = "utoipa", schema(nullable = false))]
    pub waiting_room: Option<bool>,

    /// Patch whether the event is encrypted
    #[cfg_attr(
        feature = "serde",
        serde(default, skip_serializing_if = "Option::is_none")
    )]
    // Field is non-required already, utoipa adds a `nullable: true` entry
    // by default which creates a false positive in the spectral linter when
    // combined with example data.
    #[cfg_attr(feature = "utoipa", schema(nullable = false))]
    pub e2e_encryption: Option<bool>,

    /// Patch the adhoc flag.
    #[cfg_attr(
        feature = "serde",
        serde(default, skip_serializing_if = "Option::is_none")
    )]
    // Field is non-required already, utoipa adds a `nullable: true` entry
    // by default which creates a false positive in the spectral linter when
    // combined with example data.
    #[cfg_attr(feature = "utoipa", schema(nullable = false))]
    pub is_adhoc: Option<bool>,

    /// Patch the time independence of the event
    ///
    /// If it changes the independence from true false this body has to have
    /// `is_all_day`, `starts_at` and `ends_at` set
    #[cfg_attr(
        feature = "serde",
        serde(default, skip_serializing_if = "Option::is_none")
    )]
    // Field is non-required already, utoipa adds a `nullable: true` entry
    // by default which creates a false positive in the spectral linter when
    // combined with example data.
    #[cfg_attr(feature = "utoipa", schema(nullable = false))]
    pub is_time_independent: Option<bool>,

    /// Patch if the event is an all-day event
    ///
    /// If it changes the value from false to true this request must ensure
    /// that the `starts_at.datetime` and `ends_at.datetime` have a 00:00 time part.
    #[cfg_attr(
        feature = "serde",
        serde(default, skip_serializing_if = "Option::is_none")
    )]
    // Field is non-required already, utoipa adds a `nullable: true` entry
    // by default which creates a false positive in the spectral linter when
    // combined with example data.
    #[cfg_attr(feature = "utoipa", schema(nullable = false))]
    pub is_all_day: Option<bool>,

    /// Patch the start time of the event
    #[cfg_attr(
        feature = "serde",
        serde(default, skip_serializing_if = "Option::is_none")
    )]
    // Field is non-required already, utoipa adds a `nullable: true` entry
    // by default which creates a false positive in the spectral linter when
    // combined with example data.
    #[cfg_attr(feature = "utoipa", schema(nullable = false))]
    pub starts_at: Option<DateTimeTz>,

    /// Patch the end time of the event
    #[cfg_attr(
        feature = "serde",
        serde(default, skip_serializing_if = "Option::is_none")
    )]
    // Field is non-required already, utoipa adds a `nullable: true` entry
    // by default which creates a false positive in the spectral linter when
    // combined with example data.
    #[cfg_attr(feature = "utoipa", schema(nullable = false))]
    pub ends_at: Option<DateTimeTz>,

    /// Patch the events recurrence patterns
    ///
    /// If this list is non empty it override the events current one
    #[cfg_attr(
        feature = "serde",
        serde(default, skip_serializing_if = "RecurrencePattern::is_empty")
    )]
    // Field is non-required already, utoipa adds a `nullable: true` entry
    // by default which creates a false positive in the spectral linter when
    // combined with example data.
    #[cfg_attr(feature = "utoipa", schema(nullable = false))]
    pub recurrence_pattern: RecurrencePattern,

    /// The streaming targets of the room associated with the event
    #[cfg_attr(
        feature = "serde",
        serde(default, skip_serializing_if = "Option::is_none")
    )]
    // Field is non-required already, utoipa adds a `nullable: true` entry
    // by default which creates a false positive in the spectral linter when
    // combined with example data.
    #[cfg_attr(feature = "utoipa", schema(nullable = false))]
    pub streaming_targets: Option<Vec<StreamingTarget>>,

    /// Patch wether the meeting details are displayed or not
    #[cfg_attr(
        feature = "serde",
        serde(default, skip_serializing_if = "Option::is_none")
    )]
    // Field is non-required already, utoipa adds a `nullable: true` entry
    // by default which creates a false positive in the spectral linter when
    // combined with example data.
    #[cfg_attr(feature = "utoipa", schema(nullable = false))]
    pub show_meeting_details: Option<bool>,

    /// Either add a shared folder to the event, if none existed before or delete the shared folder
    #[cfg_attr(
        feature = "serde",
        serde(default, skip_serializing_if = "Option::is_none")
    )]
    // Field is non-required already, utoipa adds a `nullable: true` entry
    // by default which creates a false positive in the spectral linter when
    // combined with example data.
    #[cfg_attr(feature = "utoipa", schema(nullable = false))]
    pub has_shared_folder: Option<bool>,
}

impl PatchEventBody {
    /// Check if the body is empty
    pub fn is_empty(&self) -> bool {
        let PatchEventBody {
            title,
            description,
            password,
            waiting_room,
            e2e_encryption,
            is_adhoc,
            is_time_independent,
            is_all_day,
            starts_at,
            ends_at,
            recurrence_pattern,
            show_meeting_details,
            has_shared_folder,
            streaming_targets,
        } = self;

        title.is_none()
            && description.is_none()
            && password.is_none()
            && waiting_room.is_none()
            && e2e_encryption.is_none()
            && is_adhoc.is_none()
            && is_time_independent.is_none()
            && is_all_day.is_none()
            && starts_at.is_none()
            && ends_at.is_none()
            && recurrence_pattern.is_empty()
            && show_meeting_details.is_none()
            && has_shared_folder.is_none()
            && streaming_targets.is_none()
    }

    // special case to only patch the events room
    /// Check if the body only modifies the room
    pub fn only_modifies_room(&self) -> bool {
        let PatchEventBody {
            title,
            description,
            password,
            waiting_room,
            e2e_encryption,
            is_time_independent,
            is_all_day,
            starts_at,
            ends_at,
            recurrence_pattern,
            is_adhoc,
            show_meeting_details,
            has_shared_folder,
            streaming_targets,
        } = self;

        title.is_none()
            && description.is_none()
            && is_time_independent.is_none()
            && is_all_day.is_none()
            && starts_at.is_none()
            && ends_at.is_none()
            && recurrence_pattern.is_empty()
            && is_adhoc.is_none()
            && show_meeting_details.is_none()
            && has_shared_folder.is_none()
            && streaming_targets.is_none()
            && (password.is_some() || waiting_room.is_some() || e2e_encryption.is_some())
    }
}

impl ExampleData for PatchEventBody {
    fn example_data() -> Self {
        Self {
            title: Some("The new title".to_string()),
            description: Some("The new description".to_string()),
            password: None,
            waiting_room: None,
            e2e_encryption: None,
            is_adhoc: None,
            is_time_independent: None,
            is_all_day: None,
            starts_at: None,
            ends_at: None,
            recurrence_pattern: RecurrencePattern::default(),
            streaming_targets: None,
            show_meeting_details: Some(false),
            has_shared_folder: Some(true),
        }
    }
}
