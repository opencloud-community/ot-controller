// SPDX-FileCopyrightText: OpenTalk GmbH <mail@opentalk.eu>
//
// SPDX-License-Identifier: EUPL-1.2

//! Signaling commands for the `breakout` namespace

use crate::{core::ParticipantId, imports::*};

/// Parameters used for starting a breakout room
#[derive(Clone, Debug, PartialEq, Eq)]
#[cfg_attr(feature = "serde", derive(Serialize, Deserialize))]
pub struct RoomParameter {
    /// Name of the breakout room
    pub name: String,
    /// Ids of participants to be assigned to the breakout room
    pub assignments: Vec<ParticipantId>,
}
