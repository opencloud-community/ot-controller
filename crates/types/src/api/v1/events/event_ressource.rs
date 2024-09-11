// SPDX-FileCopyrightText: OpenTalk GmbH <mail@opentalk.eu>
//
// SPDX-License-Identifier: EUPL-1.2

use chrono::{TimeZone as _, Utc};
use opentalk_types_common::{
    events::{invites::EventInviteStatus, EventId},
    shared_folders::SharedFolder,
    streaming::RoomStreamingTarget,
    time::{DateTimeTz, RecurrencePattern, Timestamp},
    utils::ExampleData,
};

use super::{EventInvitee, EventRoomInfo, EventType};
use crate::api::v1::users::PublicUserProfile;
#[allow(unused_imports)]
use crate::imports::*;

/// Event Resource representation
///
/// Returned from `GET /events/` and `GET /events/{event_id}`
#[derive(Debug, Clone, PartialEq, Eq)]
#[cfg_attr(feature = "serde", derive(Serialize, Deserialize))]
#[cfg_attr(
    feature = "utoipa",
    derive(utoipa::ToSchema),
    schema(example = json!(EventResource::example_data()))
)]
pub struct EventResource {
    /// ID of the event
    pub id: EventId,

    /// Public user profile of the user which created the event
    pub created_by: PublicUserProfile,

    /// Timestamp of the event creation
    pub created_at: Timestamp,

    /// Public user profile of the user which last updated the event
    pub updated_by: PublicUserProfile,

    /// Timestamp of the last update
    pub updated_at: Timestamp,

    /// Title of the event
    ///
    /// For display purposes
    pub title: String,

    /// Description of the event
    ///
    /// For display purposes
    pub description: String,

    /// All information about the room the event takes place in
    pub room: EventRoomInfo,

    /// Flag which indicates if `invitees` contains all invites as far as known to the application
    /// May also be true if there are no invitees but no invitees were requested
    pub invitees_truncated: bool,

    /// List of event invitees and their invite status. Might not be complete, see `invite_truncated`
    pub invitees: Vec<EventInvitee>,

    /// Is the event time independent?
    ///
    /// Time independent events are not bound to any time but instead are constantly available to join
    pub is_time_independent: bool,

    /// Is the event an all day event
    ///
    /// All-day events have no start/end time, they last the entire day(s)
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
    ///
    /// Omitted if `is_time_independent` is true
    ///
    /// For events of type `recurring` the datetime contains the time of the first instance.
    /// The datetimes of subsequent recurrences are computed using the datetime of the first instance and its timezone.
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
    ///
    /// Omitted if `is_time_independent` is true
    ///
    /// For events of type `recurring` the datetime contains the time of the first instance.
    /// The datetimes of subsequent recurrences are computed using the datetime of the first instance and its timezone.
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
    /// To get all event instances resolved use the the `GET /events/{event_id}/instances` endpoint.
    /// Changing this field will always remove all exceptions for the event.
    ///
    /// Allowed are `RRULE`, `RDATE`, `EXRULE` and `EXDATE`.
    ////
    /// The `DTSTART` and `DTEND` attributes are not allowed as their information is stored
    /// in the `starts_at` and `ends_at` fields.
    #[cfg_attr(
        feature = "serde",
        serde(skip_serializing_if = "RecurrencePattern::is_empty")
    )]
    pub recurrence_pattern: RecurrencePattern,

    /// Flag indicating whether the event is ad-hoc created.
    pub is_adhoc: bool,

    /// Type of event
    ///
    /// Time independent events or events without recurrence are `single` while recurring events are `recurring`
    #[cfg_attr(feature = "serde", serde(rename = "type"))]
    pub type_: EventType,

    /// The invite status of the current user for this event
    pub invite_status: EventInviteStatus,

    /// Is this event in the current user's favorite list?
    pub is_favorite: bool,

    /// Can the current user edit this resource
    pub can_edit: bool,

    /// Information about the shared folder for the event
    #[cfg_attr(
        feature = "serde",
        serde(default, skip_serializing_if = "Option::is_none")
    )]
    // Field is non-required already, utoipa adds a `nullable: true` entry
    // by default which creates a false positive in the spectral linter when
    // combined with example data.
    #[cfg_attr(feature = "utoipa", schema(nullable = false))]
    pub shared_folder: Option<SharedFolder>,

    /// The streaming targets of the room associated with the event
    #[cfg_attr(
        feature = "serde",
        serde(default, skip_serializing_if = "Vec::is_empty")
    )]
    pub streaming_targets: Vec<RoomStreamingTarget>,

    /// Indicates whether meeting details should be provided. If absent, no meeting details are made available.
    pub show_meeting_details: bool,
}

impl ExampleData for EventResource {
    fn example_data() -> Self {
        Self {
            id: EventId::example_data(),
            created_by: PublicUserProfile::example_data(),
            created_at: Timestamp::example_data(),
            updated_by: PublicUserProfile::example_data(),
            updated_at: Timestamp::example_data(),
            title: "Teammeeting".to_string(),
            description: "The weekly teammeting".to_string(),
            room: EventRoomInfo::example_data(),
            invitees_truncated: false,
            invitees: vec![EventInvitee::example_data()],
            is_time_independent: false,
            is_all_day: None,
            starts_at: Some(DateTimeTz {
                datetime: Utc.with_ymd_and_hms(2024, 7, 5, 17, 2, 42).unwrap(),
                timezone: chrono_tz::Europe::Berlin.into(),
            }),
            ends_at: None,
            recurrence_pattern: RecurrencePattern::default(),
            is_adhoc: false,
            type_: EventType::Single,
            invite_status: EventInviteStatus::Accepted,
            is_favorite: false,
            can_edit: false,
            shared_folder: Some(SharedFolder::example_data()),
            streaming_targets: vec![RoomStreamingTarget::example_data()],
            show_meeting_details: true,
        }
    }
}
