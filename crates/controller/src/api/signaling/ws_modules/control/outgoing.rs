// SPDX-FileCopyrightText: OpenTalk GmbH <mail@opentalk.eu>
//
// SPDX-License-Identifier: EUPL-1.2

use serde::Serialize;
use types::signaling::{
    control::{
        event::{Error, JoinBlockedReason, JoinSuccess},
        AssociatedParticipant, Participant,
    },
    Role,
};

#[derive(Clone, Debug, Serialize, PartialEq, Eq)]
#[serde(tag = "message", rename_all = "snake_case")]
pub enum ControlEvent {
    JoinSuccess(JoinSuccess),
    /// Joining the room failed
    JoinBlocked(JoinBlockedReason),
    /// State change of this participant
    Update(Participant),
    /// A participant that joined the room
    Joined(Participant),
    /// This participant left the room
    Left(AssociatedParticipant),
    /// The quota's time limit has elapsed
    TimeLimitQuotaElapsed,

    RoleUpdated {
        new_role: Role,
    },

    RoomDeleted,

    Error(Error),
}

#[cfg(test)]
mod test {
    use std::str::FromStr;

    use super::*;
    use chrono::DateTime;
    use db_storage::tariffs::Tariff;
    use pretty_assertions::assert_eq;
    use serde_json::json;
    use types::{
        common::tariff::TariffResource,
        core::{ParticipantId, TariffId},
    };

    fn participant_tariff() -> TariffResource {
        TariffResource {
            id: TariffId::nil(),
            name: "test".into(),
            quotas: Default::default(),
            enabled_modules: Default::default(),
        }
    }

    #[test]
    fn tariff_to_participant_tariff() {
        let tariff = Tariff {
            id: TariffId::nil(),
            name: "test".into(),
            created_at: Default::default(),
            updated_at: Default::default(),
            quotas: Default::default(),
            disabled_modules: vec![
                "whiteboard".to_string(),
                "timer".to_string(),
                "media".to_string(),
                "polls".to_string(),
            ],
        };
        let available_modules = vec!["chat", "media", "polls", "whiteboard", "timer"];

        let expected = json!({
            "id": "00000000-0000-0000-0000-000000000000",
            "name": "test",
            "quotas": {},
            "enabled_modules": ["chat"],
        });

        let actual = serde_json::to_value(tariff.to_tariff_resource(&available_modules)).unwrap();

        assert_eq!(actual, expected);
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
    fn left() {
        let expected = json!({"message": "left","id": "00000000-0000-0000-0000-000000000000"});

        let produced = serde_json::to_value(&ControlEvent::Left(AssociatedParticipant {
            id: ParticipantId::nil(),
        }))
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
