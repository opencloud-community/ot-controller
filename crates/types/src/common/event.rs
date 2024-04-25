// SPDX-FileCopyrightText: OpenTalk GmbH <mail@opentalk.eu>
//
// SPDX-License-Identifier: EUPL-1.2

//! Common types related to event

use crate::core::{EventId, InviteCodeId, RoomId};
use url::Url;

#[allow(unused_imports)]
use crate::imports::*;

/// Information about an event
#[derive(Clone, Debug, PartialEq, Eq)]
#[cfg_attr(feature = "serde", derive(Serialize, Deserialize))]
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
    #[cfg_attr(feature = "serde", serde(skip_serializing_if = "Option::is_none"))]
    pub meeting_details: Option<MeetingDetails>,
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

/// Call-in info for an event
#[derive(Debug, Clone, Eq, PartialEq)]
#[cfg_attr(feature = "serde", derive(Serialize, Deserialize))]
pub struct CallIn {
    /// SIP Call-In phone number which must be used to reach the room
    pub tel: String,

    /// SIP ID which must transmitted via DTMF (number field on the phone) to identify this room
    pub id: String,

    /// SIP password which must be transmitted via DTMF (number field on the phone) after entering the `sip_id`
    /// to enter the room
    pub password: String,
}

/// Streaming link for an event
#[derive(Debug, Clone, Eq, PartialEq)]
#[cfg_attr(feature = "serde", derive(Serialize, Deserialize))]
pub struct StreamingLink {
    /// The name of the streaming link
    pub name: String,

    /// The url of the streaming link
    pub url: Url,
}

/// Details about an event
#[derive(Clone, Debug, PartialEq, Eq)]
#[cfg_attr(feature = "serde", derive(Serialize, Deserialize))]
pub struct MeetingDetails {
    /// The invite code id of the event
    #[cfg_attr(feature = "serde", serde(skip_serializing_if = "Option::is_none"))]
    pub invite_code_id: Option<InviteCodeId>,

    /// The call-in information for the event
    #[cfg_attr(feature = "serde", serde(skip_serializing_if = "Option::is_none"))]
    pub call_in: Option<CallIn>,

    /// The links for accessing the stream
    pub streaming_links: Vec<StreamingLink>,
}
