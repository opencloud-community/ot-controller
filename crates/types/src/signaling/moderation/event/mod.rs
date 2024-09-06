// SPDX-FileCopyrightText: OpenTalk GmbH <mail@opentalk.eu>
//
// SPDX-License-Identifier: EUPL-1.2

//! Signaling events for the `moderation` namespace

mod debriefing_started;
mod raise_hands_disabled;
mod raise_hands_enabled;
mod raised_hand_reset_by_moderator;
mod session_ended;

pub use debriefing_started::DebriefingStarted;
use opentalk_types_signaling::{AssociatedParticipant, Participant, ParticipantId};
pub use raise_hands_disabled::RaiseHandsDisabled;
pub use raise_hands_enabled::RaiseHandsEnabled;
pub use raised_hand_reset_by_moderator::RaisedHandResetByModerator;
pub use session_ended::SessionEnded;

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

    /// Sent to a participant that is moved to the waiting room
    SentToWaitingRoom,

    /// Sent out when a session is ended by a moderator
    SessionEnded(SessionEnded),

    /// Sent out when debriefing of a session started
    DebriefingStarted(DebriefingStarted),

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
    RaiseHandsEnabled(RaiseHandsEnabled),

    /// Sent out when raise hands is disabled
    RaiseHandsDisabled(RaiseHandsDisabled),

    /// Sent to a participant when they are accepted by the moderator from the waiting room
    Accepted,

    /// Sent to all participants when a participants display name gets changed
    DisplayNameChanged(DisplayNameChanged),

    /// An error happened when executing a `moderation` command
    Error(Error),

    /// Sent out when raised hand is reset by a moderator
    RaisedHandResetByModerator(RaisedHandResetByModerator),
}

impl From<SessionEnded> for ModerationEvent {
    fn from(value: SessionEnded) -> Self {
        Self::SessionEnded(value)
    }
}

impl From<DebriefingStarted> for ModerationEvent {
    fn from(value: DebriefingStarted) -> Self {
        Self::DebriefingStarted(value)
    }
}

impl From<RaiseHandsEnabled> for ModerationEvent {
    fn from(value: RaiseHandsEnabled) -> Self {
        Self::RaiseHandsEnabled(value)
    }
}

impl From<RaiseHandsDisabled> for ModerationEvent {
    fn from(value: RaiseHandsDisabled) -> Self {
        Self::RaiseHandsDisabled(value)
    }
}

impl From<RaisedHandResetByModerator> for ModerationEvent {
    fn from(value: RaisedHandResetByModerator) -> Self {
        Self::RaisedHandResetByModerator(value)
    }
}

impl From<DisplayNameChanged> for ModerationEvent {
    fn from(value: DisplayNameChanged) -> Self {
        Self::DisplayNameChanged(value)
    }
}

impl From<Error> for ModerationEvent {
    fn from(value: Error) -> Self {
        Self::Error(value)
    }
}

/// Received by all participants when a participant gets their display name changed
#[derive(Clone, Debug, PartialEq, Eq)]
#[cfg_attr(
    feature = "serde",
    derive(Serialize, Deserialize),
    serde(rename_all = "snake_case")
)]
pub struct DisplayNameChanged {
    /// The participant that got their display name changed
    pub target: ParticipantId,
    /// The issuer of the display name change
    pub issued_by: ParticipantId,
    /// The old display name
    pub old_name: String,
    /// The new display name
    pub new_name: String,
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
    /// Cannot send the room owner to the waiting room
    CannotSendRoomOwnerToWaitingRoom,
    /// Cannot change the display name of registered users
    CannotChangeNameOfRegisteredUsers,
    /// Invalid display name
    InvalidDisplayName,
    /// Insufficient permissions to perform a command
    InsufficientPermissions,
}

#[cfg(test)]
mod tests {
    use opentalk_types_signaling::ParticipantId;
    use pretty_assertions::assert_eq;
    use serde_json::json;

    use super::*;

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

        let produced = serde_json::to_value(ModerationEvent::SessionEnded(SessionEnded {
            issued_by: ParticipantId::nil(),
        }))
        .unwrap();

        assert_eq!(expected, produced);
    }

    #[test]
    fn debriefing_started() {
        let expected = json!({
            "message": "debriefing_started",
            "issued_by": "00000000-0000-0000-0000-000000000000"
        });

        let produced =
            serde_json::to_value(ModerationEvent::DebriefingStarted(DebriefingStarted {
                issued_by: ParticipantId::nil(),
            }))
            .unwrap();

        assert_eq!(expected, produced);
    }

    #[test]
    fn in_waiting_room() {
        let expected = json!({"message": "in_waiting_room"});

        let produced = serde_json::to_value(ModerationEvent::InWaitingRoom).unwrap();

        assert_eq!(expected, produced);
    }

    #[test]
    fn display_name_changed() {
        let expected = json!({
            "message": "display_name_changed",
            "target": "00000000-0000-0000-0000-000000000000",
            "issued_by": "00000000-0000-0000-0000-000000000000",
            "old_name": "Alice",
            "new_name": "Bob"
        });

        let produced =
            serde_json::to_value(ModerationEvent::DisplayNameChanged(DisplayNameChanged {
                target: ParticipantId::nil(),
                issued_by: ParticipantId::nil(),
                old_name: "Alice".into(),
                new_name: "Bob".into(),
            }))
            .unwrap();

        assert_eq!(expected, produced);
    }
}
