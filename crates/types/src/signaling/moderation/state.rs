// SPDX-FileCopyrightText: OpenTalk GmbH <mail@opentalk.eu>
//
// SPDX-License-Identifier: EUPL-1.2

//! Frontend data for `moderation` namespace

use crate::{imports::*, signaling::control::Participant};

/// Moderation module state that is visible only to moderators
#[derive(Clone, Debug, PartialEq, Eq)]
#[cfg_attr(feature = "serde", derive(Serialize, Deserialize))]
pub struct ModeratorFrontendData {
    /// Is waiting room enabled
    pub waiting_room_enabled: bool,

    /// Are there participants in the waiting room
    pub waiting_room_participants: Vec<Participant>,
}
