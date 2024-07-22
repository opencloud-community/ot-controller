// SPDX-FileCopyrightText: OpenTalk GmbH <mail@opentalk.eu>
//
// SPDX-License-Identifier: EUPL-1.2

use chrono::{TimeZone as _, Utc};

#[cfg(feature = "serde")]
use crate::api::v1::utils::validate_recurrence_pattern;
#[allow(unused_imports)]
use crate::imports::*;
use crate::{
    common::streaming::StreamingTarget,
    core::{DateTimeTz, RoomPassword},
    utils::ExampleData,
};

/// Body of the `POST /events` endpoint
#[derive(Debug, Clone, PartialEq, Eq)]
#[cfg_attr(feature = "serde", derive(Serialize, Deserialize, Validate))]
#[cfg_attr(feature = "utoipa", derive(utoipa::ToSchema), schema(
    example = json!(
        PostEventsBody::example_data()
    )
))]
pub struct PostEventsBody {
    /// Title of the event
    #[cfg_attr(feature = "serde", validate(length(max = 255)))]
    pub title: String,

    /// Description of the event
    #[cfg_attr(feature = "serde", validate(length(max = 4096)))]
    pub description: String,

    /// Optional password for the room related to the event
    #[cfg_attr(
        feature = "serde",
        serde(default, skip_serializing_if = "Option::is_none")
    )]
    // Field is non-required already, utoipa adds a `nullable: true` entry
    // by default which creates a false positive in the spectral linter when
    // combined with example data.
    #[cfg_attr(feature = "utoipa", schema(nullable = false))]
    pub password: Option<RoomPassword>,

    /// Should the created event have a waiting room?
    #[cfg_attr(feature = "serde", serde(default))]
    pub waiting_room: bool,

    /// Should the created event be time independent?
    ///
    /// If true, all following fields must be null
    /// If false, requires `is_all_day`, `starts_at`, `ends_at`
    pub is_time_independent: bool,

    /// Should the event be all-day?
    ///
    /// If true, requires `starts_at.datetime` and `ends_at.datetime` to have a 00:00 time part
    #[cfg_attr(
        feature = "serde",
        serde(default, skip_serializing_if = "Option::is_none")
    )]
    // Field is non-required already, utoipa adds a `nullable: true` entry
    // by default which creates a false positive in the spectral linter when
    // combined with example data.
    #[cfg_attr(feature = "utoipa", schema(nullable = false))]
    pub is_all_day: Option<bool>,

    /// Start time of the event
    ///
    /// For recurring events these must contain the datetime of the first instance
    #[cfg_attr(
        feature = "serde",
        serde(default, skip_serializing_if = "Option::is_none")
    )]
    // Field is non-required already, utoipa adds a `nullable: true` entry
    // by default which creates a false positive in the spectral linter when
    // combined with example data.
    #[cfg_attr(feature = "utoipa", schema(nullable = false))]
    pub starts_at: Option<DateTimeTz>,

    /// End time of the event
    ///
    /// For recurring events these must contain the datetime of the first instance
    #[cfg_attr(
        feature = "serde",
        serde(default, skip_serializing_if = "Option::is_none")
    )]
    // Field is non-required already, utoipa adds a `nullable: true` entry
    // by default which creates a false positive in the spectral linter when
    // combined with example data.
    #[cfg_attr(feature = "utoipa", schema(nullable = false))]
    pub ends_at: Option<DateTimeTz>,

    /// List of recurrence patterns
    ///
    /// If the list if non-empty the created event will be of type `recurring`
    ///
    /// For more infos see the documentation of [`EventResource`]
    ///
    /// [`EventResource`]: ../event_ressource/struct.EventResource.html
    #[cfg_attr(
        feature = "serde",
        serde(default),
        validate(custom(function = "validate_recurrence_pattern"))
    )]
    pub recurrence_pattern: Vec<String>,

    /// Is this an ad-hoc chatroom?
    #[cfg_attr(feature = "serde", serde(default))]
    pub is_adhoc: bool,

    /// The streaming targets of the room associated with the event
    #[cfg_attr(
        feature = "serde",
        serde(default, skip_serializing_if = "Vec::is_empty")
    )]
    pub streaming_targets: Vec<StreamingTarget>,

    /// Should the created event have a shared folder?
    #[cfg_attr(feature = "serde", serde(default))]
    pub has_shared_folder: bool,

    /// Should it be able to show the meeting details?
    #[cfg_attr(feature = "serde", serde(default))]
    pub show_meeting_details: bool,
}

impl ExampleData for PostEventsBody {
    fn example_data() -> Self {
        Self {
            title: "Teammeeting".to_string(),
            description: "The weekly teammeeting".to_string(),
            password: Some(RoomPassword::example_data()),
            waiting_room: false,
            is_time_independent: false,
            is_all_day: None,
            starts_at: Some(DateTimeTz {
                datetime: Utc.with_ymd_and_hms(2024, 7, 22, 10, 0, 0).unwrap(),
                timezone: chrono_tz::Europe::Berlin.into(),
            }),
            ends_at: Some(DateTimeTz {
                datetime: Utc.with_ymd_and_hms(2024, 7, 22, 11, 0, 0).unwrap(),
                timezone: chrono_tz::Europe::Berlin.into(),
            }),
            recurrence_pattern: vec!["FREQ=WEEKLY;INTERVAL=1;BYDAY=MO".to_string()],
            is_adhoc: false,
            streaming_targets: vec![StreamingTarget::example_data()],
            has_shared_folder: false,
            show_meeting_details: true,
        }
    }
}
