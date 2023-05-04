// SPDX-FileCopyrightText: OpenTalk GmbH <mail@opentalk.eu>
//
// SPDX-License-Identifier: EUPL-1.2

//! Signaling commands for the `recording` namespace

#[allow(unused_imports)]
use crate::imports::*;

use super::RecordingId;

/// Commands for the `recording` namespace
#[derive(Clone, Debug, PartialEq, Eq)]
#[cfg_attr(
    feature = "serde",
    derive(Serialize, Deserialize),
    serde(tag = "action", rename_all = "snake_case")
)]
pub enum RecordingCommand {
    /// Start a recording
    Start,

    /// Stop a recording
    Stop(Stop),

    /// Set the consent status for a specific recording
    SetConsent(SetConsent),
}

/// Data for the `stop` recording command
#[derive(Clone, Debug, PartialEq, Eq)]
#[cfg_attr(feature = "serde", derive(Serialize, Deserialize))]
pub struct Stop {
    /// Id of the recording to be stopped
    pub recording_id: RecordingId,
}

/// Data for the `set_consent` recording command
#[derive(Clone, Debug, PartialEq, Eq)]
#[cfg_attr(feature = "serde", derive(Serialize, Deserialize))]
pub struct SetConsent {
    /// Flag indicating whether the participant consents to being recorded
    pub consent: bool,
}
