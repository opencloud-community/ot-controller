// SPDX-FileCopyrightText: OpenTalk GmbH <mail@opentalk.eu>
//
// SPDX-License-Identifier: EUPL-1.2

//! Signaling events for the `breakout` namespace

use crate::core::{BreakoutRoomId, Timestamp};

#[allow(unused_imports)]
use crate::imports::*;

use super::{AssociatedParticipantInOtherRoom, BreakoutRoom, ParticipantInOtherRoom};

/// Events sent out by the `breakout` module
#[derive(Clone, Debug, PartialEq, Eq)]
#[cfg_attr(
    feature = "serde",
    derive(Serialize, Deserialize),
    serde(tag = "message", rename_all = "snake_case")
)]
pub enum BreakoutEvent {
    /// The breakout session has started
    Started(Started),

    /// The breakout session has stopped
    Stopped,

    /// The breakout session has expired
    Expired,

    /// Another participant has joined another breakout room in the session
    Joined(ParticipantInOtherRoom),

    /// A participant has left another breakout room in the session
    Left(AssociatedParticipantInOtherRoom),

    /// An error happened when executing a `breakout` command
    Error(Error),
}

/// Event signaling to the participant that the breakout session has started
#[derive(Clone, Debug, PartialEq, Eq)]
#[cfg_attr(feature = "serde", derive(Serialize, Deserialize))]
pub struct Started {
    /// List of the breakout rooms
    pub rooms: Vec<BreakoutRoom>,
    /// The expiration time of the breakout session
    pub expires: Option<Timestamp>,
    /// The id of the assigned breakout room
    pub assignment: Option<BreakoutRoomId>,
}

impl From<Started> for BreakoutEvent {
    fn from(value: Started) -> Self {
        Self::Started(value)
    }
}

/// Error from the `breakout` module namespace
#[derive(Clone, Debug, PartialEq, Eq)]
#[cfg_attr(
    feature = "serde",
    derive(Serialize, Deserialize),
    serde(tag = "error", rename_all = "snake_case")
)]
pub enum Error {
    ///  No active breakout session is running
    Inactive,
    /// Insufficient permissions to perform a command
    InsufficientPermissions,
}

impl From<Error> for BreakoutEvent {
    fn from(value: Error) -> Self {
        Self::Error(value)
    }
}

#[cfg(test)]
mod test {
    use super::*;
    use crate::{
        core::{BreakoutRoomId, ParticipantId, ParticipationKind, Timestamp},
        signaling::{breakout::BreakoutRoom, Role},
    };
    use pretty_assertions::assert_eq;
    use serde_json::json;

    #[test]
    fn started() {
        let expected = json!({
            "message": "started",
            "rooms": [
                {"id":"00000000-0000-0000-0000-000000000000", "name":"Room 1"},
                {"id":"00000000-0000-0000-0000-000000000001","name":"Room 2"},
            ],
            "expires": null,
            "assignment": "00000000-0000-0000-0000-000000000000",
        });

        let produced = serde_json::to_value(BreakoutEvent::Started(Started {
            rooms: vec![
                BreakoutRoom {
                    id: BreakoutRoomId::from_u128(0),
                    name: "Room 1".into(),
                },
                BreakoutRoom {
                    id: BreakoutRoomId::from_u128(1),
                    name: "Room 2".into(),
                },
            ],
            expires: None,
            assignment: Some(BreakoutRoomId::nil()),
        }))
        .unwrap();

        assert_eq!(expected, produced);
    }

    #[test]
    fn stopped() {
        let expected = json!({"message": "stopped"});

        let produced = serde_json::to_value(BreakoutEvent::Stopped).unwrap();

        assert_eq!(expected, produced);
    }

    #[test]
    fn expired() {
        let expected = json!({"message": "expired"});

        let produced = serde_json::to_value(BreakoutEvent::Expired).unwrap();

        assert_eq!(expected, produced);
    }

    #[test]
    fn joined() {
        assert_eq!(
            serde_json::to_value(BreakoutEvent::Joined(ParticipantInOtherRoom {
                breakout_room: Some(BreakoutRoomId::nil()),
                id: ParticipantId::nil(),
                display_name: "test".into(),
                role: Role::Moderator,
                avatar_url: Some("example.org/avatar.png".into()),
                participation_kind: ParticipationKind::User,
                joined_at: Timestamp::unix_epoch(),
                left_at: None,
            }))
            .unwrap(),
            json!({
                "message": "joined",
                "breakout_room": "00000000-0000-0000-0000-000000000000",
                "id": "00000000-0000-0000-0000-000000000000",
                "display_name": "test",
                "role": "moderator",
                "avatar_url": "example.org/avatar.png",
                "participation_kind": "user",
                "joined_at": "1970-01-01T00:00:00Z",
            })
        );
    }

    #[test]
    fn left() {
        let expected = json!({
            "message": "left",
            "breakout_room": "00000000-0000-0000-0000-000000000000",
            "id": "00000000-0000-0000-0000-000000000000",
        });

        let produced =
            serde_json::to_value(BreakoutEvent::Left(AssociatedParticipantInOtherRoom {
                breakout_room: Some(BreakoutRoomId::nil()),
                id: ParticipantId::nil(),
            }))
            .unwrap();

        assert_eq!(expected, produced);
    }

    #[test]
    fn error() {
        let expected = json!({"message": "error", "error": "insufficient_permissions"});

        let produced =
            serde_json::to_value(BreakoutEvent::Error(Error::InsufficientPermissions)).unwrap();

        assert_eq!(expected, produced);
    }
}
