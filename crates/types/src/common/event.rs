// SPDX-FileCopyrightText: OpenTalk GmbH <mail@opentalk.eu>
//
// SPDX-License-Identifier: EUPL-1.2

//! Common types related to event

use opentalk_types_common::{events::EventId, utils::ExampleData};
use url::Url;

use crate::core::{InviteCodeId, RoomId};
#[allow(unused_imports)]
use crate::imports::*;

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
    pub e2e_encrytion: bool,
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
            e2e_encrytion: false,
        }
    }
}

/// Call-in info for an event
#[derive(Debug, Clone, Eq, PartialEq)]
#[cfg_attr(feature = "serde", derive(Serialize, Deserialize))]
#[cfg_attr(
    feature = "utoipa",
    derive(utoipa::ToSchema),
    schema(example = json!(CallIn::example_data()))
)]
pub struct CallIn {
    /// SIP Call-In phone number which must be used to reach the room
    pub tel: String,

    /// SIP ID which must transmitted via DTMF (number field on the phone) to identify this room
    pub id: String,

    /// SIP password which must be transmitted via DTMF (number field on the phone) after entering the `sip_id`
    /// to enter the room
    pub password: String,
}

impl ExampleData for CallIn {
    fn example_data() -> Self {
        Self {
            tel: "+555-123-456-789".to_string(),
            id: "1234567890".to_string(),
            password: "0987654321".to_string(),
        }
    }
}

/// Streaming link for an event
#[derive(Debug, Clone, Eq, PartialEq)]
#[cfg_attr(feature = "serde", derive(Serialize, Deserialize))]
#[cfg_attr(
    feature = "utoipa",
    derive(utoipa::ToSchema),
    schema(example = json!(StreamingLink::example_data()))
)]
pub struct StreamingLink {
    /// The name of the streaming link
    pub name: String,

    /// The url of the streaming link
    pub url: Url,
}

impl ExampleData for StreamingLink {
    fn example_data() -> Self {
        Self {
            name: "My OwnCast Stream".to_string(),
            url: "https://owncast.example.com/mystream".parse().unwrap(),
        }
    }
}

/// Details about an event
#[derive(Clone, Debug, PartialEq, Eq)]
#[cfg_attr(feature = "serde", derive(Serialize, Deserialize))]
#[cfg_attr(
    feature = "utoipa",
    derive(utoipa::ToSchema),
    schema(example = json!(MeetingDetails::example_data()))
)]
pub struct MeetingDetails {
    /// The invite code id of the event
    #[cfg_attr(
        feature = "serde",
        serde(default, skip_serializing_if = "Option::is_none")
    )]
    // Field is non-required already, utoipa adds a `nullable: true` entry
    // by default which creates a false positive in the spectral linter when
    // combined with example data.
    #[cfg_attr(feature = "utoipa", schema(nullable = false))]
    pub invite_code_id: Option<InviteCodeId>,

    /// The call-in information for the event
    #[cfg_attr(
        feature = "serde",
        serde(default, skip_serializing_if = "Option::is_none")
    )]
    // Field is non-required already, utoipa adds a `nullable: true` entry
    // by default which creates a false positive in the spectral linter when
    // combined with example data.
    #[cfg_attr(feature = "utoipa", schema(nullable = false))]
    pub call_in: Option<CallIn>,

    /// The links for accessing the stream
    #[cfg_attr(feature = "serde", serde(default))]
    pub streaming_links: Vec<StreamingLink>,
}

impl ExampleData for MeetingDetails {
    fn example_data() -> Self {
        Self {
            invite_code_id: Some(InviteCodeId::example_data()),
            call_in: Some(CallIn::example_data()),
            streaming_links: vec![StreamingLink::example_data()],
        }
    }
}
