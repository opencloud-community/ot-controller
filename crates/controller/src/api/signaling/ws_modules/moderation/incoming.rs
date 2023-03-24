// SPDX-FileCopyrightText: OpenTalk GmbH <mail@opentalk.eu>
//
// SPDX-License-Identifier: EUPL-1.2

use serde::Deserialize;
use types::core::ParticipantId;

use super::KickScope;

#[derive(Debug, Deserialize)]
#[serde(tag = "action", rename_all = "snake_case")]
pub enum Message {
    Kick(Target),
    Ban(Target),

    Debrief(KickScope),

    EnableWaitingRoom,
    DisableWaitingRoom,

    EnableRaiseHands,
    DisableRaiseHands,

    Accept(Target),

    ResetRaisedHands,
}

#[derive(Debug, Deserialize)]
pub struct Target {
    /// The participant to ban/kick from the room
    pub target: ParticipantId,
}

#[cfg(test)]
mod test {
    use super::*;
    use pretty_assertions::assert_eq;
    use serde_json::json;

    #[test]
    fn kick() {
        let json = json!({
            "action": "kick",
            "target": "00000000-0000-0000-0000-000000000000"
        });

        let msg: Message = serde_json::from_value(json).unwrap();

        if let Message::Kick(Target { target }) = msg {
            assert_eq!(target, ParticipantId::nil());
        } else {
            panic!()
        }
    }

    #[test]
    fn ban() {
        let json = json!({
            "action": "ban",
            "target": "00000000-0000-0000-0000-000000000000"
        });

        let msg: Message = serde_json::from_value(json).unwrap();

        if let Message::Ban(Target { target }) = msg {
            assert_eq!(target, ParticipantId::nil());
        } else {
            panic!()
        }
    }

    #[test]
    fn debrief() {
        let json = json!({
            "action": "debrief",
            "kick_scope": "users_and_guests"
        });

        let msg: Message = serde_json::from_value(json).unwrap();

        if let Message::Debrief(KickScope::UsersAndGuests) = msg {
        } else {
            panic!()
        }
    }

    #[test]
    fn accept() {
        let json = json!({
            "action": "accept",
            "target": "00000000-0000-0000-0000-000000000000"
        });

        let msg: Message = serde_json::from_value(json).unwrap();

        if let Message::Accept(Target { target }) = msg {
            assert_eq!(target, ParticipantId::nil());
        } else {
            panic!()
        }
    }
}
