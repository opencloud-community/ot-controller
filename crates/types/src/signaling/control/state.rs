// SPDX-FileCopyrightText: OpenTalk GmbH <mail@opentalk.eu>
//
// SPDX-License-Identifier: EUPL-1.2

//! Frontend data for `control` namespace

use crate::{
    core::{ParticipationKind, Timestamp},
    signaling::Role,
};

#[allow(unused_imports)]
use crate::imports::*;

/// The state of a participant in the `control` module.
///
/// This struct is sent to the participant in the `join_success` message
/// when they join successfully to the meeting.
#[derive(Clone, Debug, PartialEq, Eq)]
#[cfg_attr(feature = "serde", derive(Serialize, Deserialize))]
pub struct ControlState {
    /// Display name of the participant
    pub display_name: String,

    /// Role of the participant
    pub role: Role,

    /// The URL to the avatar of the participant
    #[cfg_attr(feature = "serde", serde(skip_serializing_if = "Option::is_none"))]
    pub avatar_url: Option<String>,

    /// The kind of participation in the meeting
    pub participation_kind: ParticipationKind,

    /// If the participant's hand is raised
    pub hand_is_up: bool,

    /// The timestamp when the participant joined the meeting
    pub joined_at: Timestamp,

    /// The timestamp when the participant left the meeting
    pub left_at: Option<Timestamp>,

    /// The timestamp when the hand raise was last updated
    pub hand_updated_at: Timestamp,

    /// If the participant is the room owner
    #[cfg_attr(feature = "serde", serde(default))]
    pub is_room_owner: bool,
}

#[cfg(feature = "serde")]
impl SignalingModulePeerFrontendData for ControlState {
    const NAMESPACE: Option<&'static str> = Some(super::NAMESPACE);
}
