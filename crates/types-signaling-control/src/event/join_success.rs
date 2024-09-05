// SPDX-FileCopyrightText: OpenTalk GmbH <mail@opentalk.eu>
//
// SPDX-License-Identifier: EUPL-1.2

use opentalk_types_common::{events::EventInfo, tariffs::TariffResource, time::Timestamp};
use opentalk_types_signaling::{Participant, ParticipantId, Role};

use crate::room::RoomInfo;

/// The data received by a participant upon successfully joining a meeting
#[derive(Clone, Debug, PartialEq, Eq)]
#[cfg_attr(feature = "serde", derive(serde::Serialize, serde::Deserialize))]
pub struct JoinSuccess {
    /// The id of the participant who joined
    pub id: ParticipantId,

    /// The display name of the participant who joined
    pub display_name: String,

    /// The URL to the avatar of the participant who joined
    #[cfg_attr(feature = "serde", serde(skip_serializing_if = "Option::is_none"))]
    pub avatar_url: Option<String>,

    /// The role of the participant in the meeting
    pub role: Role,

    /// The timestamp when the meeting will close
    #[cfg_attr(feature = "serde", serde(skip_serializing_if = "Option::is_none"))]
    pub closes_at: Option<Timestamp>,

    /// The tariff of the meeting
    pub tariff: Box<TariffResource>,

    /// The module data for the participant
    #[cfg(feature = "serde")]
    #[serde(flatten)]
    pub module_data: opentalk_types_signaling::ModuleData,

    /// List of participants in the meeting
    pub participants: Vec<Participant>,

    /// Information about the event which is associated with the room
    #[cfg_attr(feature = "serde", serde(default))]
    pub event_info: Option<EventInfo>,

    /// Information about the current room
    pub room_info: RoomInfo,

    /// Flag indicating if the participant is the room owner
    #[cfg_attr(feature = "serde", serde(default))]
    pub is_room_owner: bool,
}

impl JoinSuccess {
    /// Gets the inner module of a JoinSuccess Message
    #[cfg(feature = "serde")]
    pub fn get_module<T: opentalk_types_signaling::SignalingModuleFrontendData>(
        &self,
    ) -> Result<Option<T>, serde_json::Error> {
        self.module_data.get()
    }
}
