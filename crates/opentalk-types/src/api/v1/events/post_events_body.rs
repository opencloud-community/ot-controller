// SPDX-FileCopyrightText: OpenTalk GmbH <mail@opentalk.eu>
//
// SPDX-License-Identifier: EUPL-1.2

use chrono::{TimeZone as _, Utc};
use opentalk_types_common::{
    events::{EventDescription, EventTitle},
    rooms::RoomPassword,
    streaming::StreamingTarget,
    time::{DateTimeTz, RecurrencePattern},
    utils::ExampleData,
};

#[allow(unused_imports)]
use crate::imports::*;

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
    pub title: EventTitle,

    /// Description of the event
    pub description: EventDescription,

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

    /// Should the created event be encrypted?
    #[cfg_attr(feature = "serde", serde(default))]
    pub e2e_encryption: bool,

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

    /// Recurrence pattern(s) for recurring events
    ///
    /// May contain RRULE, EXRULE, RDATE and EXDATE strings
    ///
    /// Requires `type` to be `recurring`
    ///
    /// Contains a list of recurrence rules which describe the occurrences of the event.
    ///
    /// Allowed are `RRULE`, `RDATE`, `EXRULE` and `EXDATE`.
    ////
    /// The `DTSTART` and `DTEND` attributes are not allowed as their information is stored
    /// in the `starts_at` and `ends_at` fields.
    #[cfg_attr(
        feature = "serde",
        serde(default, skip_serializing_if = "RecurrencePattern::is_empty")
    )]
    pub recurrence_pattern: RecurrencePattern,

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
            title: "Teammeeting".parse().expect("valid event title"),
            description: "The weekly teammeeting"
                .parse()
                .expect("valid event description"),
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
            recurrence_pattern: RecurrencePattern::example_data(),
            is_adhoc: false,
            streaming_targets: vec![StreamingTarget::example_data()],
            has_shared_folder: false,
            show_meeting_details: true,
            e2e_encryption: false,
        }
    }
}
