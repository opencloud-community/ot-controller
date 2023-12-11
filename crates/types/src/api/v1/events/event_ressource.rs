// SPDX-FileCopyrightText: OpenTalk GmbH <mail@opentalk.eu>
//
// SPDX-License-Identifier: EUPL-1.2

#[allow(unused_imports)]
use crate::imports::*;
use crate::{
    api::v1::users::PublicUserProfile,
    common::{shared_folder::SharedFolder, streaming::RoomStreamingTarget},
    core::{DateTimeTz, EventId, EventInviteStatus, Timestamp},
};

use super::{EventInvitee, EventRoomInfo, EventType};

/// Event Resource representation
///
/// Returned from `GET /events/` and `GET /events/{event_id}`
#[derive(Debug, Clone, PartialEq, Eq)]
#[cfg_attr(feature = "serde", derive(Serialize, Deserialize))]
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
    #[cfg_attr(feature = "serde", serde(skip_serializing_if = "Option::is_none"))]
    pub is_all_day: Option<bool>,

    /// Start time of the event.
    ///
    /// Omitted if `is_time_independent` is true
    ///
    /// For events of type `recurring` the datetime contains the time of the first instance.
    /// The datetimes of subsequent recurrences are computed using the datetime of the first instance and its timezone.
    #[cfg_attr(feature = "serde", serde(skip_serializing_if = "Option::is_none"))]
    pub starts_at: Option<DateTimeTz>,

    /// End time of the event.
    ///
    /// Omitted if `is_time_independent` is true
    ///
    /// For events of type `recurring` the datetime contains the time of the first instance.
    /// The datetimes of subsequent recurrences are computed using the datetime of the first instance and its timezone.
    #[cfg_attr(feature = "serde", serde(skip_serializing_if = "Option::is_none"))]
    pub ends_at: Option<DateTimeTz>,

    /// Recurrence pattern(s) for recurring events
    ///
    /// May contain RRULE, EXRULE, RDATE and EXDATE strings
    ///
    /// Requires `type` to be `recurring`
    #[cfg_attr(feature = "serde", serde(skip_serializing_if = "Vec::is_empty"))]
    pub recurrence_pattern: Vec<String>,

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
    #[cfg_attr(feature = "serde", serde(skip_serializing_if = "Option::is_none"))]
    pub shared_folder: Option<SharedFolder>,

    /// The streaming targets of the room associated with the event
    #[cfg_attr(
        feature = "serde",
        serde(default, skip_serializing_if = "Option::is_none")
    )]
    pub streaming_targets: Option<Vec<RoomStreamingTarget>>,
}
