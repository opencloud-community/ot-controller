// SPDX-FileCopyrightText: OpenTalk GmbH <mail@opentalk.eu>
//
// SPDX-License-Identifier: EUPL-1.2

use serde::Deserialize;
use types::signaling::common::TargetParticipant;

#[derive(Debug, Deserialize)]
#[serde(tag = "action", rename_all = "snake_case")]
pub enum Message {
    Join(Join),
    /// Enter into the room while being in the waiting room
    /// after being accepted by a moderator
    EnterRoom,
    RaiseHand,
    LowerHand,
    GrantModeratorRole(TargetParticipant),
    RevokeModeratorRole(TargetParticipant),
}

#[derive(Debug, Deserialize)]
pub struct Join {
    /// The users display name
    pub display_name: String,
}

#[cfg(test)]
mod test {
    use super::*;
    use pretty_assertions::assert_eq;

    #[test]
    fn hello() {
        let json = r#"
        {
            "action": "join",
            "display_name": "Test!"
        }
        "#;

        let msg: Message = serde_json::from_str(json).unwrap();

        if let Message::Join(Join { display_name }) = msg {
            assert_eq!(display_name, "Test!");
        } else {
            panic!()
        }
    }

    #[test]
    fn raise_hand() {
        let json = r#"
        {
            "action": "raise_hand"
        }
        "#;

        let msg: Message = serde_json::from_str(json).unwrap();

        assert!(matches!(msg, Message::RaiseHand));
    }

    #[test]
    fn lower_hand() {
        let json = r#"
        {
            "action": "lower_hand"
        }
        "#;

        let msg: Message = serde_json::from_str(json).unwrap();

        assert!(matches!(msg, Message::LowerHand));
    }
}
