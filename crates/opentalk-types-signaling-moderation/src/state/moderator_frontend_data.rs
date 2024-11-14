// SPDX-FileCopyrightText: OpenTalk GmbH <mail@opentalk.eu>
//
// SPDX-License-Identifier: EUPL-1.2

use opentalk_types_signaling::Participant;

/// Moderation module state that is visible only to moderators
#[derive(Clone, Debug, PartialEq, Eq)]
#[cfg_attr(feature = "serde", derive(serde::Serialize, serde::Deserialize))]
pub struct ModeratorFrontendData {
    /// Is waiting room enabled
    pub waiting_room_enabled: bool,

    /// Are there participants in the waiting room
    pub waiting_room_participants: Vec<Participant>,
}
