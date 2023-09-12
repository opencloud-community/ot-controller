// SPDX-FileCopyrightText: OpenTalk GmbH <mail@opentalk.eu>
//
// SPDX-License-Identifier: EUPL-1.2

//! Signaling events for the `moderation` namespace

use crate::{
    core::ParticipantId,
    signaling::control::{AssociatedParticipant, Participant},
};

#[allow(unused_imports)]
use crate::imports::*;

/// Events sent out by the `moderation` module
#[derive(Clone, Debug, PartialEq, Eq)]
#[cfg_attr(
    feature = "serde",
    derive(Serialize, Deserialize),
    serde(tag = "message", rename_all = "snake_case")
)]
pub enum ModerationEvent {
    /// Sent to a participant when they are kicked from a meeting
    Kicked,

    /// Sent to a participant when they are banned from a meeting
    Banned,

    /// Sent out when a session is ended by a moderator
    SessionEnded {
        /// The moderator who ended the session
        issued_by: ParticipantId,
    },

    /// Sent out when debriefing of a session started
    DebriefingStarted {
        /// The moderator who started the debriefing
        issued_by: ParticipantId,
    },

    /// Sent to participants who are placed into a waiting room
    InWaitingRoom,

    /// Sent to the moderator when a participant joined the waiting room
    JoinedWaitingRoom(Participant),

    /// Sent to the moderator when a participant left the waiting room
    LeftWaitingRoom(AssociatedParticipant),

    /// Sent out when the waiting room is enabled
    WaitingRoomEnabled,

    /// Sent out when the waiting room is disabled
    WaitingRoomDisabled,

    /// Sent out when raise hands is enabled
    RaiseHandsEnabled {
        /// The moderator who enabled raise hands
        issued_by: ParticipantId,
    },

    /// Sent out when raise hands is disabled
    RaiseHandsDisabled {
        /// The moderator who disabled raise hands
        issued_by: ParticipantId,
    },

    /// Sent to a participant when they are accepted by the moderator from the waiting room
    Accepted,

    /// An error happened when executing a `moderation` command
    Error(Error),

    /// Sent out when raised hand is reset by a moderator
    RaisedHandResetByModerator {
        /// The moderator who reset raised hand
        issued_by: ParticipantId,
    },
}

/// Error from the `moderation` module namespace
#[derive(Clone, Debug, PartialEq, Eq)]
#[cfg_attr(
    feature = "serde",
    derive(Serialize, Deserialize),
    serde(tag = "error", rename_all = "snake_case")
)]
pub enum Error {
    /// Cannot ban a guest participant
    CannotBanGuest,
}

impl From<Error> for ModerationEvent {
    fn from(value: Error) -> Self {
        Self::Error(value)
    }
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
