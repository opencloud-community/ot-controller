// SPDX-FileCopyrightText: OpenTalk GmbH <mail@opentalk.eu>
//
// SPDX-License-Identifier: EUPL-1.2

use serde::Deserialize;
use types::{core::ParticipantId, signaling::moderation::KickScope};

#[derive(Debug, Deserialize)]
#[serde(tag = "action", rename_all = "snake_case")]
pub enum ModerationCommand {
    Kick {
        /// The participant to kick from the room
        target: ParticipantId,
    },
    Ban {
        /// The participant to ban from the room
        target: ParticipantId,
    },

    Debrief(KickScope),

    EnableWaitingRoom,
    DisableWaitingRoom,

    EnableRaiseHands,
    DisableRaiseHands,

    Accept {
        /// The participant to accept into the room
        target: ParticipantId,
    },

    ResetRaisedHands,
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

        let msg: ModerationCommand = serde_json::from_value(json).unwrap();

        if let ModerationCommand::Kick { target } = msg {
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

        let msg: ModerationCommand = serde_json::from_value(json).unwrap();

        if let ModerationCommand::Ban { target } = msg {
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

        let msg: ModerationCommand = serde_json::from_value(json).unwrap();

        if let ModerationCommand::Debrief(KickScope::UsersAndGuests) = msg {
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

        let msg: ModerationCommand = serde_json::from_value(json).unwrap();

        if let ModerationCommand::Accept { target } = msg {
            assert_eq!(target, ParticipantId::nil());
        } else {
            panic!()
        }
    }
}
