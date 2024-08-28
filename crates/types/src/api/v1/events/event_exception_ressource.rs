// SPDX-FileCopyrightText: OpenTalk GmbH <mail@opentalk.eu>
//
// SPDX-License-Identifier: EUPL-1.2

use chrono::{TimeZone as _, Utc};

use super::{EventAndInstanceId, EventStatus, EventType, InstanceId};
#[allow(unused_imports)]
use crate::imports::*;
use crate::{
    api::v1::users::PublicUserProfile,
    core::{DateTimeTz, EventId, Timestamp},
    utils::ExampleData,
};

/// Event exception resource
///
/// Overrides event properties for a event recurrence. May only exist for events of type `recurring`.
#[derive(Debug, Clone, PartialEq, Eq)]
#[cfg_attr(feature = "serde", derive(Serialize, Deserialize))]
#[cfg_attr(
    feature = "utoipa",
    derive(utoipa::ToSchema),
    schema(example = json!(EventExceptionResource::example_data()))
)]
pub struct EventExceptionResource {
    /// Opaque ID of the exception
    pub id: EventAndInstanceId,

    /// ID of the event  the exception belongs to
    pub recurring_event_id: EventId,

    /// ID of the instance the exception overrides
    pub instance_id: InstanceId,

    /// Public user profile of the user which created the exception
    pub created_by: PublicUserProfile,

    /// Timestamp of the exceptions creation
    pub created_at: Timestamp,

    /// Public user profile of the user which last updated the exception
    pub updated_by: PublicUserProfile,

    /// Timestamp of the exceptions last update
    pub updated_at: Timestamp,

    /// Override the title of the instance
    #[cfg_attr(
        feature = "serde",
        serde(default, skip_serializing_if = "Option::is_none")
    )]
    // Field is non-required already, utoipa adds a `nullable: true` entry
    // by default which creates a false positive in the spectral linter when
    // combined with example data.
    #[cfg_attr(feature = "utoipa", schema(nullable = false))]
    pub title: Option<String>,

    /// Override the description of the instance
    #[cfg_attr(
        feature = "serde",
        serde(default, skip_serializing_if = "Option::is_none")
    )]
    // Field is non-required already, utoipa adds a `nullable: true` entry
    // by default which creates a false positive in the spectral linter when
    // combined with example data.
    #[cfg_attr(feature = "utoipa", schema(nullable = false))]
    pub description: Option<String>,

    /// Override the `is_all_day` property of the instance
    #[cfg_attr(
        feature = "serde",
        serde(default, skip_serializing_if = "Option::is_none")
    )]
    // Field is non-required already, utoipa adds a `nullable: true` entry
    // by default which creates a false positive in the spectral linter when
    // combined with example data.
    #[cfg_attr(feature = "utoipa", schema(nullable = false))]
    pub is_all_day: Option<bool>,

    /// Override the `starts_at` time of the instance
    #[cfg_attr(
        feature = "serde",
        serde(default, skip_serializing_if = "Option::is_none")
    )]
    // Field is non-required already, utoipa adds a `nullable: true` entry
    // by default which creates a false positive in the spectral linter when
    // combined with example data.
    #[cfg_attr(feature = "utoipa", schema(nullable = false))]
    pub starts_at: Option<DateTimeTz>,

    /// Override the `ends_at` time of the instance
    #[cfg_attr(
        feature = "serde",
        serde(default, skip_serializing_if = "Option::is_none")
    )]
    // Field is non-required already, utoipa adds a `nullable: true` entry
    // by default which creates a false positive in the spectral linter when
    // combined with example data.
    #[cfg_attr(feature = "utoipa", schema(nullable = false))]
    pub ends_at: Option<DateTimeTz>,

    /// The `starts_at` of the instance this exception modifies. Used to match the exception the instance
    pub original_starts_at: DateTimeTz,

    /// Must always be `exception`
    #[cfg_attr(feature = "serde", serde(rename = "type"))]
    pub type_: EventType,

    /// Override the status of the event instance
    ///
    /// This can be used to cancel a occurrence of an event
    pub status: EventStatus,

    /// Can the current user edit this resource
    pub can_edit: bool,
}

impl ExampleData for EventExceptionResource {
    fn example_data() -> Self {
        Self {
            id: EventAndInstanceId::example_data(),
            recurring_event_id: EventId::example_data(),
            instance_id: InstanceId::example_data(),
            created_by: PublicUserProfile::example_data(),
            created_at: Timestamp::example_data(),
            updated_by: PublicUserProfile::example_data(),
            updated_at: Timestamp::example_data(),
            title: Some("Teammeeting".to_string()),
            description: Some("The weekly teammeeting".to_string()),
            is_all_day: Some(false),
            starts_at: Some(DateTimeTz {
                datetime: Utc.with_ymd_and_hms(2024, 7, 5, 15, 0, 0).unwrap(),
                timezone: chrono_tz::Europe::Berlin.into(),
            }),
            ends_at: Some(DateTimeTz {
                datetime: Utc.with_ymd_and_hms(2024, 7, 5, 17, 0, 0).unwrap(),
                timezone: chrono_tz::Europe::Berlin.into(),
            }),
            original_starts_at: DateTimeTz {
                datetime: Utc.with_ymd_and_hms(2024, 7, 5, 16, 0, 0).unwrap(),
                timezone: chrono_tz::Europe::Berlin.into(),
            },
            type_: EventType::Instance,
            status: EventStatus::Ok,
            can_edit: false,
        }
    }
}
