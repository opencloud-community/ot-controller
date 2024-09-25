// SPDX-FileCopyrightText: OpenTalk GmbH <mail@opentalk.eu>
//
// SPDX-License-Identifier: EUPL-1.2

//! Common types related to event

#[allow(unused_imports)]
use crate::imports::*;
use crate::{
    events::{EventId, MeetingDetails},
    rooms::RoomId,
    utils::ExampleData,
};

/// Information about an event
#[derive(Clone, Debug, PartialEq, Eq)]
#[cfg_attr(feature = "serde", derive(Serialize, Deserialize))]
#[cfg_attr(
    feature = "utoipa",
    derive(utoipa::ToSchema),
    schema(example = json!(EventInfo::example_data())),
)]
pub struct EventInfo {
    /// The id of the event
    pub id: EventId,

    /// The id of the room belonging to the event
    pub room_id: RoomId,

    /// The title of the event
    pub title: String,

    /// True if the event was created ad-hoc
    pub is_adhoc: bool,

    /// The meeting details of the event
    // Field is non-required already, utoipa adds a `nullable: true` entry
    // by default which creates a false positive in the spectral linter when
    // combined with example data.
    #[cfg_attr(feature = "utoipa", schema(nullable = false))]
    #[cfg_attr(feature = "serde", serde(skip_serializing_if = "Option::is_none"))]
    pub meeting_details: Option<MeetingDetails>,

    /// Indicates whether the meeting room should have e2e encryption enabled.
    pub e2e_encryption: bool,
}

impl EventInfo {
    /// Enriches the event info with meeting details
    pub fn with_meeting_details(self, meeting_details: MeetingDetails) -> EventInfo {
        EventInfo {
            meeting_details: Some(meeting_details),
            ..self
        }
    }
}

impl ExampleData for EventInfo {
    fn example_data() -> Self {
        Self {
            id: EventId::example_data(),
            room_id: RoomId::example_data(),
            title: "Weekly Team Event".to_string(),
            is_adhoc: false,
            meeting_details: Some(MeetingDetails::example_data()),
            e2e_encryption: false,
        }
    }
}
