// SPDX-FileCopyrightText: OpenTalk GmbH <mail@opentalk.eu>
//
// SPDX-License-Identifier: EUPL-1.2

//! Signaling commands for the `breakout` namespace

use std::time::Duration;

use crate::{core::ParticipantId, imports::*};

/// Command to start a breakout session
#[derive(Clone, Debug, PartialEq, Eq)]
#[cfg_attr(feature = "serde", derive(Serialize, Deserialize))]
pub struct Start {
    /// A list of breakout rooms to create
    pub rooms: Vec<RoomParameter>,

    /// Duration of the breakout session
    #[cfg_attr(
        feature = "serde",
        serde(
            default,
            skip_serializing_if = "Option::is_none",
            with = "crate::utils::duration_seconds_option"
        )
    )]
    pub duration: Option<Duration>,
}

/// Parameters used for starting a breakout room
#[derive(Clone, Debug, PartialEq, Eq)]
#[cfg_attr(feature = "serde", derive(Serialize, Deserialize))]
pub struct RoomParameter {
    /// Name of the breakout room
    pub name: String,
    /// Ids of participants to be assigned to the breakout room
    pub assignments: Vec<ParticipantId>,
}
