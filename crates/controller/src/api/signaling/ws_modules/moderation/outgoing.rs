// SPDX-FileCopyrightText: OpenTalk GmbH <mail@opentalk.eu>
//
// SPDX-License-Identifier: EUPL-1.2

use serde::Serialize;
use types::{
    core::ParticipantId,
    signaling::control::{AssociatedParticipant, Participant},
};

#[derive(Debug, Serialize, PartialEq, Eq)]
#[serde(tag = "message", rename_all = "snake_case")]
pub enum ModerationEvent {
    Kicked,
    Banned,

    SessionEnded { issued_by: ParticipantId },
    DebriefingStarted { issued_by: ParticipantId },

    InWaitingRoom,

    JoinedWaitingRoom(Participant),
    LeftWaitingRoom(AssociatedParticipant),

    WaitingRoomEnabled,
    WaitingRoomDisabled,

    RaiseHandsEnabled { issued_by: ParticipantId },
    RaiseHandsDisabled { issued_by: ParticipantId },

    Accepted,

    Error(Error),

    RaisedHandResetByModerator { issued_by: ParticipantId },
}

#[derive(Debug, Serialize, PartialEq, Eq)]
#[serde(tag = "error", rename_all = "snake_case")]
pub enum Error {
    CannotBanGuest,
}

#[cfg(test)]
mod test {
    use super::*;
    use pretty_assertions::assert_eq;
    use serde_json::json;

    #[test]
    fn kicked() {
        let expected = json!({"message": "kicked"});

        let produced = serde_json::to_value(ModerationEvent::Kicked).unwrap();

        assert_eq!(expected, produced);
    }

    #[test]
    fn banned() {
        let expected = json!({"message": "banned"});

        let produced = serde_json::to_value(ModerationEvent::Banned).unwrap();

        assert_eq!(expected, produced);
    }

    #[test]
    fn session_ended() {
        let expected = json!({
            "message": "session_ended",
            "issued_by": "00000000-0000-0000-0000-000000000000"
        });

        let produced = serde_json::to_value(ModerationEvent::SessionEnded {
            issued_by: ParticipantId::nil(),
        })
        .unwrap();

        assert_eq!(expected, produced);
    }

    #[test]
    fn debriefing_started() {
        let expected = json!({
            "message": "debriefing_started",
            "issued_by": "00000000-0000-0000-0000-000000000000"
        });

        let produced = serde_json::to_value(ModerationEvent::DebriefingStarted {
            issued_by: ParticipantId::nil(),
        })
        .unwrap();

        assert_eq!(expected, produced);
    }

    #[test]
    fn in_waiting_room() {
        let expected = json!({"message": "in_waiting_room"});

        let produced = serde_json::to_value(ModerationEvent::InWaitingRoom).unwrap();

        assert_eq!(expected, produced);
    }
}
