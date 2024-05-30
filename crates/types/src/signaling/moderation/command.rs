// SPDX-FileCopyrightText: OpenTalk GmbH <mail@opentalk.eu>
//
// SPDX-License-Identifier: EUPL-1.2

//! Signaling commands for the `moderation` namespace

use super::KickScope;
use crate::core::ParticipantId;
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

    /// Same behavior as the Kick command, but implies different handling from the client
    SendToWaitingRoom {
        /// The participant to move to the waiting room
        target: ParticipantId,
    },

    /// Start the debriefing
    Debrief(KickScope),

    /// Change the display name of the targeted guest
    ChangeDisplayName {
        /// The new display name
        new_name: String,
        /// The participant that will have their name changed
        target: ParticipantId,
    },

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
    ResetRaisedHands {
        /// An optional single participant to reset the raised hand for
        target: Option<ParticipantId>,
    },
}

#[cfg(test)]
mod test {
    use pretty_assertions::assert_eq;
    use serde_json::json;

    use super::*;

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

    #[test]
    fn reset_raised_hand_for_single_participant() {
        let json = json!({
            "action": "reset_raised_hands",
            "target": "00000000-0000-0000-0000-000000000000"
        });

        let msg: ModerationCommand = serde_json::from_value(json).unwrap();

        if let ModerationCommand::ResetRaisedHands { target } = msg {
            assert_eq!(target, Some(ParticipantId::nil()));
        } else {
            panic!()
        }
    }

    #[test]
    fn reset_raised_hands_for_all_participants() {
        let json = json!({
            "action": "reset_raised_hands"
        });

        let msg: ModerationCommand = serde_json::from_value(json).unwrap();

        if let ModerationCommand::ResetRaisedHands { target } = msg {
            assert_eq!(target, None);
        } else {
            panic!()
        }
    }
}
