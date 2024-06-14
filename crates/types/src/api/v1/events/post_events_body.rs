// SPDX-FileCopyrightText: OpenTalk GmbH <mail@opentalk.eu>
//
// SPDX-License-Identifier: EUPL-1.2

#[cfg(feature = "serde")]
use crate::api::v1::utils::validate_recurrence_pattern;
#[allow(unused_imports)]
use crate::imports::*;
use crate::{
    common::streaming::StreamingTarget,
    core::{DateTimeTz, RoomPassword},
};

/// Body of the `POST /events` endpoint
#[derive(Debug, Clone, PartialEq, Eq)]
#[cfg_attr(feature = "serde", derive(Serialize, Deserialize, Validate))]
pub struct PostEventsBody {
    /// Title of the event
    #[cfg_attr(feature = "serde", validate(length(max = 255)))]
    pub title: String,

    /// Description of the event
    #[cfg_attr(feature = "serde", validate(length(max = 4096)))]
    pub description: String,

    /// Optional password for the room related to the event
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
    pub is_all_day: Option<bool>,

    /// Start time of the event
    ///
    /// For recurring events these must contain the datetime of the first instance
    pub starts_at: Option<DateTimeTz>,

    /// End time of the event
    ///
    /// For recurring events these must contain the datetime of the first instance
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
