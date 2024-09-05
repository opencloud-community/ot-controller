// SPDX-FileCopyrightText: OpenTalk GmbH <mail@opentalk.eu>
//
// SPDX-License-Identifier: EUPL-1.2

//! Types related to signaling events in the `control` namespace

use opentalk_types_signaling::{Participant, Role, TargetParticipant};
use opentalk_types_signaling_control::event::{JoinBlockedReason, JoinSuccess, Left};

#[allow(unused_imports)]
use crate::imports::*;

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
    Left(Left),
    /// The quota's time limit has elapsed
    TimeLimitQuotaElapsed,

    /// This participant raised a hand
    HandRaised,
    /// This participant lowered a hand
    HandLowered,

    /// This participant's role in the meeting has been updated
    RoleUpdated(RoleUpdated),

    /// The room has been deleted
    RoomDeleted,

    /// An error happened when executing a `control` command
    Error(Error),

    /// The moderator role has been granted to another participant
    ModeratorRoleGranted(TargetParticipant),
    /// The moderator role has been revoked from another participant
    ModeratorRoleRevoked(TargetParticipant),
}

impl From<JoinSuccess> for ControlEvent {
    fn from(value: JoinSuccess) -> Self {
        Self::JoinSuccess(value)
    }
}

impl From<Left> for ControlEvent {
    fn from(value: Left) -> Self {
        Self::Left(value)
    }
}

/// The participant role was updated.
#[derive(Clone, Debug, PartialEq, Eq)]
#[cfg_attr(feature = "serde", derive(Serialize, Deserialize))]
pub struct RoleUpdated {
    /// The new role of the participant
    pub new_role: Role,
}

impl From<RoleUpdated> for ControlEvent {
    fn from(value: RoleUpdated) -> Self {
        Self::RoleUpdated(value)
    }
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
mod tests {
    use std::str::FromStr;

    use chrono::DateTime;
    use opentalk_types_common::{
        events::{EventId, EventInfo},
        rooms::RoomId,
        tariffs::{TariffId, TariffResource},
    };
    use opentalk_types_signaling::{
        AssociatedParticipant, LeaveReason, ModulePeerData, ParticipantId,
    };
    use opentalk_types_signaling_control::room::{CreatorInfo, RoomInfo};
    use pretty_assertions::assert_eq;
    use serde_json::json;

    use super::*;

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
                "e2e_encryption": false,
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

        let produced = serde_json::to_value(ControlEvent::JoinSuccess(JoinSuccess {
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
                title: "Daily".parse().expect("valid event title"),
                is_adhoc: false,
                meeting_details: None,
                e2e_encryption: false,
            }),
            room_info: RoomInfo {
                id: RoomId::nil(),
                password: Some("secret123".parse().unwrap()),
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
                "e2e_encryption": false,
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

        let produced = serde_json::to_value(ControlEvent::JoinSuccess(JoinSuccess {
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
                title: "Daily".parse().expect("valid event title"),
                is_adhoc: false,
                meeting_details: None,
                e2e_encryption: false,
            }),
            room_info: RoomInfo {
                id: RoomId::nil(),
                password: Some("secret123".parse().unwrap()),
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
        let expected = json!({
            "message": "update",
            "id": "00000000-0000-0000-0000-000000000000",
            "dummy_namespace_1": {
                "field_1": false,
                "field_2": true,
            },
            "dummy_namespace_2": {
                "field_a": true,
                "field_b": false,
            }
        });

        let mut module_data = ModulePeerData::default();
        let _ = module_data.insert(&DummyFrontendData1 {
            field_1: false,
            field_2: true,
        });
        let _ = module_data.insert(&DummyFrontendData2 {
            field_a: true,
            field_b: false,
        });

        let produced = serde_json::to_value(ControlEvent::Update(Participant {
            id: ParticipantId::nil(),
            module_data,
        }))
        .unwrap();

        assert_eq!(expected, produced);
    }

    #[derive(Default, Debug, PartialEq, Eq, Clone)]
    #[cfg_attr(feature = "serde", derive(Serialize, Deserialize))]
    pub struct DummyFrontendData1 {
        pub field_1: bool,
        pub field_2: bool,
    }

    #[cfg(feature = "serde")]
    impl SignalingModulePeerFrontendData for DummyFrontendData1 {
        const NAMESPACE: Option<&'static str> = Some("dummy_namespace_1");
    }

    #[derive(Default, Debug, PartialEq, Eq, Clone)]
    #[cfg_attr(feature = "serde", derive(Serialize, Deserialize))]
    pub struct DummyFrontendData2 {
        pub field_a: bool,
        pub field_b: bool,
    }

    #[cfg(feature = "serde")]
    impl SignalingModulePeerFrontendData for DummyFrontendData2 {
        const NAMESPACE: Option<&'static str> = Some("dummy_namespace_2");
    }

    #[test]
    fn joined() {
        let expected = json!({"message": "joined", "id": "00000000-0000-0000-0000-000000000000"});

        let produced = serde_json::to_value(ControlEvent::Joined(Participant {
            id: ParticipantId::nil(),
            module_data: Default::default(),
        }))
        .unwrap();

        assert_eq!(expected, produced);
    }

    #[test]
    fn left_quit_reason() {
        let expected = json!({"message": "left","id": "00000000-0000-0000-0000-000000000000", "reason": "quit"});

        let produced = serde_json::to_value(ControlEvent::Left(Left {
            id: AssociatedParticipant {
                id: ParticipantId::nil(),
            },
            reason: LeaveReason::Quit,
        }))
        .unwrap();

        assert_eq!(expected, produced);
    }

    #[test]
    fn left_timeout_reason() {
        let expected = json!({"message": "left","id": "00000000-0000-0000-0000-000000000000", "reason": "timeout"});

        let produced = serde_json::to_value(ControlEvent::Left(Left {
            id: AssociatedParticipant {
                id: ParticipantId::nil(),
            },
            reason: LeaveReason::Timeout,
        }))
        .unwrap();

        assert_eq!(expected, produced);
    }

    #[test]
    fn error() {
        let expected = json!({"message": "error", "error": "raise_hands_disabled"});

        let produced =
            serde_json::to_value(ControlEvent::Error(Error::RaiseHandsDisabled)).unwrap();

        assert_eq!(expected, produced);
    }
}
