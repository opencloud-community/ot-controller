// SPDX-FileCopyrightText: OpenTalk GmbH <mail@opentalk.eu>
//
// SPDX-License-Identifier: EUPL-1.2

//! Signaling events for the `recording` namespace

use crate::imports::*;

use super::RecordingId;

/// Data for the `started` recording event
#[derive(Clone, Debug, PartialEq, Eq)]
#[cfg_attr(feature = "serde", derive(Serialize, Deserialize))]
pub struct Started {
    /// The id of the recording that was started
    pub recording_id: RecordingId,
}

/// Data for the `stopped` recording event
#[derive(Clone, Debug, PartialEq, Eq)]
#[cfg_attr(feature = "serde", derive(Serialize, Deserialize))]
pub struct Stopped {
    /// The id of the recording that was stopped
    pub recording_id: RecordingId,
}

/// Error from the `recording` module namespace
#[derive(Clone, Debug, PartialEq, Eq)]
#[cfg_attr(
    feature = "serde",
    derive(Serialize, Deserialize),
    serde(tag = "error", rename_all = "snake_case")
)]
pub enum Error {
    /// The participant has insufficient permissions to perform a command
    InsufficientPermissions,

    /// Attempted to start a recording while it is already running
    AlreadyRecording,

    /// Invalid recording id used
    InvalidRecordingId,
}
