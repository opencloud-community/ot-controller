// SPDX-FileCopyrightText: OpenTalk GmbH <mail@opentalk.eu>
//
// SPDX-License-Identifier: EUPL-1.2

//! Signaling messages for the `control` namespace

use crate::signaling::common::TargetParticipant;

#[allow(unused_imports)]
use crate::imports::*;

/// Commands received by the `control` module
#[derive(Clone, Debug, PartialEq, Eq)]
#[cfg_attr(
    feature = "serde",
    derive(Serialize, Deserialize),
    serde(tag = "action", rename_all = "snake_case")
)]
pub enum ControlCommand {
    /// Join a meeting
    Join(Join),
    /// Enter into the room while being in the waiting room
    /// after being accepted by a moderator
    EnterRoom,
    /// Raise a hand
    RaiseHand,
    /// Lower a raised hand
    LowerHand,
    /// Grant moderator role to another participant
    GrantModeratorRole(TargetParticipant),
    /// Revoke moderator role from another participant
    RevokeModeratorRole(TargetParticipant),
}

/// Body of the join command
#[derive(Clone, Debug, PartialEq, Eq)]
#[cfg_attr(feature = "serde", derive(Serialize, Deserialize))]
pub struct Join {
    /// The users display name
    #[cfg_attr(
        feature = "serde",
        serde(default, skip_serializing_if = "Option::is_none",)
    )]
    pub display_name: Option<String>,
}

#[cfg(test)]
mod test {
    use super::*;
    use pretty_assertions::assert_eq;
    use serde_json::json;

    #[test]
    fn join_with_display_name() {
        let json = json!({
            "action": "join",
            "display_name": "Test!",
        });

        let msg: ControlCommand = serde_json::from_value(json).unwrap();

        if let ControlCommand::Join(Join { display_name }) = msg {
            assert_eq!(display_name, Some("Test!".to_owned()));
        } else {
            panic!()
        }
    }

    #[test]
    fn join_without_display_name() {
        let json = json!({
            "action": "join",
        });

        let msg: ControlCommand = serde_json::from_value(json).unwrap();

        if let ControlCommand::Join(Join { display_name }) = msg {
            assert_eq!(display_name, None);
        } else {
            panic!()
        }
    }

    #[test]
    fn raise_hand() {
        let json = json!({
            "action": "raise_hand",
        });

        let msg: ControlCommand = serde_json::from_value(json).unwrap();

        assert!(matches!(msg, ControlCommand::RaiseHand));
    }

    #[test]
    fn lower_hand() {
        let json = json!({
            "action": "lower_hand",
        });

        let msg: ControlCommand = serde_json::from_value(json).unwrap();

        assert!(matches!(msg, ControlCommand::LowerHand));
    }
}
