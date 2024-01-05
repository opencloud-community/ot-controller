// SPDX-FileCopyrightText: OpenTalk GmbH <mail@opentalk.eu>
//
// SPDX-License-Identifier: EUPL-1.2

//! Frontend data for `moderation` namespace

use crate::signaling::control::Participant;

#[allow(unused_imports)]
use crate::imports::*;

/// The state of the `moderation` module.
///
/// This struct is sent to the participant in the `join_success` message
/// when they join successfully to the meeting.
#[derive(Clone, Debug, PartialEq, Eq)]
#[cfg_attr(feature = "serde", derive(Serialize, Deserialize))]
pub struct ModerationState {
    /// Moderation module data that is only avaialble for moderators
    #[cfg_attr(feature = "serde", serde(flatten))]
    pub moderator_data: Option<ModeratorFrontendData>,

    /// Is raise hands enabled
    pub raise_hands_enabled: bool,
}

#[cfg(feature = "serde")]
impl SignalingModuleFrontendData for ModerationState {
    const NAMESPACE: Option<&'static str> = Some(super::NAMESPACE);
}

/// Moderation module state that is visible only to moderators
#[derive(Clone, Debug, PartialEq, Eq)]
#[cfg_attr(feature = "serde", derive(Serialize, Deserialize))]
pub struct ModeratorFrontendData {
    /// Is waiting room enabled
    pub waiting_room_enabled: bool,

    /// Are there participants in the waiting room
    pub waiting_room_participants: Vec<Participant>,
}
