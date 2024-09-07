// SPDX-FileCopyrightText: OpenTalk GmbH <mail@opentalk.eu>
//
// SPDX-License-Identifier: EUPL-1.2

//! Signaling commands for the `moderation` namespace

mod ban;
mod change_display_name;
mod kick;
mod reset_raised_hands;
mod send_to_waiting_room;

pub use ban::Ban;
pub use change_display_name::ChangeDisplayName;
pub use kick::Kick;
use opentalk_types_signaling_moderation::{command::Accept, KickScope};
pub use reset_raised_hands::ResetRaisedHands;
pub use send_to_waiting_room::SendToWaitingRoom;

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
    Kick(Kick),

    /// Ban a participant from the room
    Ban(Ban),

    /// Send a participant to the waiting room
    SendToWaitingRoom(SendToWaitingRoom),

    /// Start the debriefing
    Debrief(KickScope),

    /// Change the display name of the targeted guest
    ChangeDisplayName(ChangeDisplayName),

    /// Enable waiting room for the meeting
    EnableWaitingRoom,

    /// Disable waiting room for the meeting
    DisableWaitingRoom,

    /// Enable raise hands for the meeting
    EnableRaiseHands,

    /// Disable raise hands for the meeting
    DisableRaiseHands,

    /// Accept a participant into the meeting
    Accept(Accept),

    /// Reset raised hands for the meeting
    ResetRaisedHands(ResetRaisedHands),
}

impl From<Kick> for ModerationCommand {
    fn from(value: Kick) -> Self {
        Self::Kick(value)
    }
}

impl From<Ban> for ModerationCommand {
    fn from(value: Ban) -> Self {
        Self::Ban(value)
    }
}

impl From<SendToWaitingRoom> for ModerationCommand {
    fn from(value: SendToWaitingRoom) -> Self {
        Self::SendToWaitingRoom(value)
    }
}

impl From<ChangeDisplayName> for ModerationCommand {
    fn from(value: ChangeDisplayName) -> Self {
        Self::ChangeDisplayName(value)
    }
}

impl From<Accept> for ModerationCommand {
    fn from(value: Accept) -> Self {
        Self::Accept(value)
    }
}

impl From<ResetRaisedHands> for ModerationCommand {
    fn from(value: ResetRaisedHands) -> Self {
        Self::ResetRaisedHands(value)
    }
}

#[cfg(test)]
mod tests {
    use std::collections::BTreeSet;

    use opentalk_types_signaling::ParticipantId;
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

        if let ModerationCommand::Kick(Kick { target }) = msg {
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

        if let ModerationCommand::Ban(Ban { target }) = msg {
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

        if let ModerationCommand::Accept(Accept { target }) = msg {
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

        if let ModerationCommand::ResetRaisedHands(ResetRaisedHands { target }) = msg {
            assert_eq!(target, Some(BTreeSet::from_iter([ParticipantId::nil()])));
        } else {
            panic!()
        }
    }

    #[test]
    fn reset_raised_hand_for_multiple_participants() {
        let json = json!({
            "action": "reset_raised_hands",
            "target": ["00000000-0000-0000-0000-000000000000", "00000000-0000-0000-0000-00000000cafe"]
        });

        let msg: ModerationCommand = serde_json::from_value(json).unwrap();

        if let ModerationCommand::ResetRaisedHands(ResetRaisedHands { target }) = msg {
            assert_eq!(
                target,
                Some(BTreeSet::from_iter([
                    ParticipantId::from_u128(0),
                    ParticipantId::from_u128(0xcafe)
                ]))
            );
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

        if let ModerationCommand::ResetRaisedHands(ResetRaisedHands { target }) = msg {
            assert_eq!(target, None);
        } else {
            panic!()
        }
    }
}
