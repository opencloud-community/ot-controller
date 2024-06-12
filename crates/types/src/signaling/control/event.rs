// SPDX-FileCopyrightText: OpenTalk GmbH <mail@opentalk.eu>
//
// SPDX-License-Identifier: EUPL-1.2

//! Types related to signaling events in the `control` namespace

use super::{reason, room::RoomInfo, AssociatedParticipant, Participant};
#[allow(unused_imports)]
use crate::imports::*;
use crate::{
    common::{event::EventInfo, tariff::TariffResource},
    core::{ParticipantId, Timestamp},
    signaling::Role,
};

/// Events sent out by the `control` module
#[derive(Clone, Debug, PartialEq, Eq)]
#[cfg_attr(
    feature = "serde",
    derive(Serialize, Deserialize),
    serde(tag = "message", rename_all = "snake_case")
)]
pub enum ControlEvent {
    /// The participant joined successfully
    JoinSuccess(JoinSuccess),
    /// Joining the room failed
    JoinBlocked(JoinBlockedReason),
    /// State change of this participant
    Update(Participant),
    /// A participant that joined the room
    Joined(Participant),
    /// This participant left the room
    Left {
        /// The participant that left
        #[cfg_attr(feature = "serde", serde(flatten))]
        id: AssociatedParticipant,
        /// The reason as to why the participant left
        reason: reason::Reason,
    },
    /// The quota's time limit has elapsed
    TimeLimitQuotaElapsed,

    /// This participant raised a hand
    HandRaised,
    /// This participant lowered a hand
    HandLowered,

    /// This participant's role in the meeting has been updated
    RoleUpdated {
        /// The new role of the participant
        new_role: Role,
    },

    /// The room has been deleted
    RoomDeleted,

    /// An error happened when executing a `control` command
    Error(Error),
}

/// The data received by a participant upon successfully joining a meeting
#[derive(Clone, Debug, PartialEq, Eq)]
#[cfg_attr(feature = "serde", derive(Serialize, Deserialize))]
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
    pub module_data: crate::signaling::ModuleData,

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

impl From<JoinSuccess> for ControlEvent {
    fn from(value: JoinSuccess) -> Self {
        Self::JoinSuccess(value)
    }
}

impl JoinSuccess {
    /// Gets the inner module of a JoinSuccess Message
    #[cfg(feature = "serde")]
    pub fn get_module<T: SignalingModuleFrontendData>(
        &self,
    ) -> Result<Option<T>, serde_json::Error> {
        self.module_data.get()
    }
}

/// The reason for blocking a participant from joining a meeting
#[derive(Clone, Debug, PartialEq, Eq)]
#[cfg_attr(
    feature = "serde",
    derive(Serialize, Deserialize),
    serde(tag = "reason", rename_all = "snake_case")
)]
pub enum JoinBlockedReason {
    /// The participant limit for the meeting's tariff has been reached
    ParticipantLimitReached,
}

/// Errors from the `control` module namespace
#[derive(Debug, Clone, PartialEq, Eq)]
#[cfg_attr(
    feature = "serde",
    derive(Serialize, Deserialize),
    serde(tag = "error", rename_all = "snake_case")
)]
pub enum Error {
    /// Payload sent to the `control` module had the wrong JSON format
    InvalidJson,

    /// Attempted to send data to a module namespace that does not exist
    InvalidNamespace,

    /// The chosen user name does not meet the requirements
    InvalidUsername,

    /// Participant attempted to join while already participating in the meeting
    AlreadyJoined,

    /// Attempted to perform a command on a participant that has not yet joined the meeting
    NotYetJoined,

    /// A participant attempted to join the meeting who is neither in the waiting room nor accepted
    NotAcceptedOrNotInWaitingRoom,

    /// Attempted to raise hand while handraising is disabled for the meeting
    RaiseHandsDisabled,

    /// Attempted to perform a command which requires more permissions
    InsufficientPermissions,

    /// Attempted to grant or revoke moderation permissions to the room owner who implicitly has these permissions anyway
    TargetIsRoomOwner,

    /// An issued command requires no further actions
    NothingToDo,
}
#[cfg(test)]
mod test {
    use std::str::FromStr;

    use chrono::DateTime;
    use pretty_assertions::assert_eq;
    use serde_json::json;

    use super::*;
    use crate::{
        core::{EventId, RoomId, TariffId},
        signaling::control::{self, room::CreatorInfo},
    };

    fn participant_tariff() -> TariffResource {
        TariffResource {
            id: TariffId::nil(),
            name: "test".into(),
            quotas: Default::default(),
            enabled_modules: Default::default(),
            disabled_features: Default::default(),
            modules: Default::default(),
        }
    }

    #[test]
    fn join_success() {
        let expected = json!({
            "message": "join_success",
            "id": "00000000-0000-0000-0000-000000000000",
            "display_name": "name",
            "avatar_url": "http://url",
            "role": "user",
            "closes_at":"2021-06-24T14:00:11.873753715Z",
            "tariff": serde_json::to_value(participant_tariff()).unwrap(),
            "participants": [],
            "event_info": {
                "id": "00000000-0000-0000-0000-000000000000",
                "room_id": "00000000-0000-0000-0000-000000000000",
                "title": "Daily",
                "is_adhoc": false,
            },
            "room_info": {
                "id": "00000000-0000-0000-0000-000000000000",
                "password": "secret123",
                "created_by": {
                    "title": "Dr.",
                    "firstname": "Bob",
                    "lastname": "Bobsen",
                    "display_name": "Bob",
                    "avatar_url": "example.org/avatar.png"
                },

            },
            "is_room_owner": false,
        });

        let produced = serde_json::to_value(&ControlEvent::JoinSuccess(JoinSuccess {
            id: ParticipantId::nil(),
            display_name: "name".into(),
            avatar_url: Some("http://url".into()),
            role: Role::User,
            closes_at: Some(
                DateTime::from_str("2021-06-24T14:00:11.873753715Z")
                    .unwrap()
                    .into(),
            ),
            tariff: participant_tariff().into(),
            module_data: Default::default(),
            participants: vec![],
            event_info: Some(EventInfo {
                id: EventId::nil(),
                room_id: RoomId::nil(),
                title: "Daily".to_string(),
                is_adhoc: false,
                meeting_details: None,
            }),
            room_info: RoomInfo {
                id: RoomId::nil(),
                password: Some("secret123".into()),
                created_by: CreatorInfo {
                    title: "Dr.".into(),
                    firstname: "Bob".into(),
                    lastname: "Bobsen".into(),
                    display_name: "Bob".into(),
                    avatar_url: "example.org/avatar.png".into(),
                },
            },
            is_room_owner: false,
        }))
        .unwrap();

        assert_eq!(expected, produced);
    }

    #[test]
    fn join_success_guest() {
        let expected = json!({
            "message": "join_success",
            "id": "00000000-0000-0000-0000-000000000000",
            "display_name": "name",
            "role": "guest",
            "tariff": serde_json::to_value(participant_tariff()).unwrap(),
            "participants": [],
            "event_info": {
                "id": "00000000-0000-0000-0000-000000000000",
                "room_id": "00000000-0000-0000-0000-000000000000",
                "title": "Daily",
                "is_adhoc": false,
            },
            "room_info": {
                "id": "00000000-0000-0000-0000-000000000000",
                "password": "secret123",
                "created_by": {
                    "title": "",
                    "firstname": "Bob",
                    "lastname": "Bobsen",
                    "display_name": "Bob",
                    "avatar_url": "example.org/avatar.png"
                },

            },
            "is_room_owner": false,
        });

        let produced = serde_json::to_value(&ControlEvent::JoinSuccess(JoinSuccess {
            id: ParticipantId::nil(),
            display_name: "name".into(),
            avatar_url: None,
            role: Role::Guest,
            closes_at: None,
            tariff: participant_tariff().into(),
            module_data: Default::default(),
            participants: vec![],
            event_info: Some(EventInfo {
                id: EventId::nil(),
                room_id: RoomId::nil(),
                title: "Daily".to_string(),
                is_adhoc: false,
                meeting_details: None,
            }),
            room_info: RoomInfo {
                id: RoomId::nil(),
                password: Some("secret123".into()),
                created_by: CreatorInfo {
                    title: "".into(),
                    firstname: "Bob".into(),
                    lastname: "Bobsen".into(),
                    display_name: "Bob".into(),
                    avatar_url: "example.org/avatar.png".into(),
                },
            },
            is_room_owner: false,
        }))
        .unwrap();

        assert_eq!(expected, produced);
    }

    #[test]
    fn update() {
        let expected = json!({"message": "update", "id": "00000000-0000-0000-0000-000000000000"});

        let produced = serde_json::to_value(&ControlEvent::Update(Participant {
            id: ParticipantId::nil(),
            module_data: Default::default(),
        }))
        .unwrap();

        assert_eq!(expected, produced);
    }

    #[test]
    fn joined() {
        let expected = json!({"message": "joined", "id": "00000000-0000-0000-0000-000000000000"});

        let produced = serde_json::to_value(&ControlEvent::Joined(Participant {
            id: ParticipantId::nil(),
            module_data: Default::default(),
        }))
        .unwrap();

        assert_eq!(expected, produced);
    }

    #[test]
    fn left_quit_reason() {
        let expected = json!({"message": "left","id": "00000000-0000-0000-0000-000000000000", "reason": "quit"});

        let produced = serde_json::to_value(&ControlEvent::Left {
            id: AssociatedParticipant {
                id: ParticipantId::nil(),
            },
            reason: control::Reason::Quit,
        })
        .unwrap();

        assert_eq!(expected, produced);
    }

    #[test]
    fn left_timeout_reason() {
        let expected = json!({"message": "left","id": "00000000-0000-0000-0000-000000000000", "reason": "timeout"});

        let produced = serde_json::to_value(&ControlEvent::Left {
            id: AssociatedParticipant {
                id: ParticipantId::nil(),
            },
            reason: control::Reason::Timeout,
        })
        .unwrap();

        assert_eq!(expected, produced);
    }

    #[test]
    fn error() {
        let expected = json!({"message": "error", "error": "raise_hands_disabled"});

        let produced =
            serde_json::to_value(&ControlEvent::Error(Error::RaiseHandsDisabled)).unwrap();

        assert_eq!(expected, produced);
    }
}
