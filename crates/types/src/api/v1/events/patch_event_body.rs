// SPDX-FileCopyrightText: OpenTalk GmbH <mail@opentalk.eu>
//
// SPDX-License-Identifier: EUPL-1.2
#[cfg(feature = "serde")]
use crate::api::v1::utils::{deserialize_some, validate_recurrence_pattern};
use crate::core::DateTimeTz;

#[allow(unused_imports)]
use crate::imports::*;

/// Body for the `PATCH /events/{event_id}` endpoint
#[derive(Debug, Clone, PartialEq, Eq)]
#[cfg_attr(feature = "serde", derive(Serialize, Deserialize, Validate))]
pub struct PatchEventBody {
    /// Patch the title of th event
    #[cfg_attr(feature = "serde", validate(length(max = 255)))]
    pub title: Option<String>,

    /// Patch the description of the event
    #[cfg_attr(feature = "serde", validate(length(max = 4096)))]
    pub description: Option<String>,

    /// Patch the password of the event's room
    #[cfg_attr(
        feature = "serde",
        serde(default, deserialize_with = "deserialize_some"),
        validate(length(min = 1, max = 255))
    )]
    pub password: Option<Option<String>>,

    /// Patch the presence of a waiting room
    pub waiting_room: Option<bool>,

    /// Patch the adhoc flag.
    pub is_adhoc: Option<bool>,

    /// Patch the time independence of the event
    ///
    /// If it changes the independence from true false this body has to have
    /// `is_all_day`, `starts_at` and `ends_at` set
    ///
    /// See documentation of [`PostEventsBody`] for more info
    ///
    /// [`PostEventsBody`]: ../post_events_body/struct.PostEventsBody.html
    pub is_time_independent: Option<bool>,

    /// Patch if the event is an all-day event
    ///
    /// If it changes the value from false to true this request must ensure
    /// that the `starts_at.datetime` and `ends_at.datetime` have a 00:00 time part.
    ///
    /// See documentation of [`PostEventsBody`] for more info
    ///
    /// [`PostEventsBody`]: ../post_events_body/struct.PostEventsBody.html
    pub is_all_day: Option<bool>,

    /// Patch the start time of the event
    pub starts_at: Option<DateTimeTz>,
    /// Patch the end time of the event
    pub ends_at: Option<DateTimeTz>,

    /// Patch the events recurrence patterns
    ///
    /// If this list is non empty it override the events current one
    #[cfg_attr(
        feature = "serde",
        serde(default),
        validate(custom = "validate_recurrence_pattern")
    )]
    pub recurrence_pattern: Vec<String>,
}

impl PatchEventBody {
    /// Check if the body is empty
    pub fn is_empty(&self) -> bool {
        let PatchEventBody {
            title,
            description,
            password,
            waiting_room,
            is_adhoc,
            is_time_independent,
            is_all_day,
            starts_at,
            ends_at,
            recurrence_pattern,
        } = self;

        title.is_none()
            && description.is_none()
            && password.is_none()
            && waiting_room.is_none()
            && is_adhoc.is_none()
            && is_time_independent.is_none()
            && is_all_day.is_none()
            && starts_at.is_none()
            && ends_at.is_none()
            && recurrence_pattern.is_empty()
    }

    // special case to only patch the events room
    /// Check if the body only modifies the room
    pub fn only_modifies_room(&self) -> bool {
        let PatchEventBody {
            title,
            description,
            password,
            waiting_room,
            is_time_independent,
            is_all_day,
            starts_at,
            ends_at,
            recurrence_pattern,
            is_adhoc,
        } = self;

        title.is_none()
            && description.is_none()
            && is_time_independent.is_none()
            && is_all_day.is_none()
            && starts_at.is_none()
            && ends_at.is_none()
            && recurrence_pattern.is_empty()
            && is_adhoc.is_none()
            && (password.is_some() || waiting_room.is_some())
    }
}
