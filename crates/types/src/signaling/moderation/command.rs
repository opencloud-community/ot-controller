// SPDX-FileCopyrightText: OpenTalk GmbH <mail@opentalk.eu>
//
// SPDX-License-Identifier: EUPL-1.2

//! Signaling commands for the `moderation` namespace

use crate::core::ParticipantId;

use super::KickScope;

#[allow(unused_imports)]
use crate::imports::*;

/// Commands for the `moderation` namespace
#[derive(Clone, Debug, PartialEq, Eq)]
#[cfg_attr(
    feature = "serde",
    derive(Serialize, Deserialize),
    serde(tag = "action", rename_all = "snake_case")
)]
pub enum ModerationCommand {
    /// Kick a participant from the room
    Kick {
        /// The participant to kick from the room
        target: ParticipantId,
    },
    /// Ban a participant from the room
    Ban {
        /// The participant to ban from the room
        target: ParticipantId,
    },

    /// Start the debriefing
    Debrief(KickScope),

    /// Enable waiting room for the meeting
    EnableWaitingRoom,

    /// Disable waiting room for the meeting
    DisableWaitingRoom,

    /// Enable raise hands for the meeting
    EnableRaiseHands,

    /// Disable raise hands for the meeting
    DisableRaiseHands,

    /// Accept a participant into the meeting
    Accept {
        /// The participant to accept into the meeting
        target: ParticipantId,
    },

    /// Reset raised hands for the meeting
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
