// SPDX-FileCopyrightText: OpenTalk GmbH <mail@opentalk.eu>
//
// SPDX-License-Identifier: EUPL-1.2

//! Signaling events for the `recording` namespace

#[allow(unused_imports)]
use crate::imports::*;

use super::RecordingId;

/// Events sent out by the `recording` module
#[derive(Clone, Debug, PartialEq, Eq)]
#[cfg_attr(
    feature = "serde",
    derive(Serialize, Deserialize),
    serde(tag = "message", rename_all = "snake_case")
)]
pub enum RecordingEvent {
    /// A recording has been started
    Started(Started),

    /// A recording has been stopped
    Stopped(Stopped),

    /// An error happened when executing a `recording` command
    Error(Error),
}

/// Data for the `started` recording event
#[derive(Clone, Debug, PartialEq, Eq)]
#[cfg_attr(feature = "serde", derive(Serialize, Deserialize))]
pub struct Started {
    /// The id of the recording that was started
    pub recording_id: RecordingId,
}

impl From<Started> for RecordingEvent {
    fn from(value: Started) -> Self {
        Self::Started(value)
    }
}

/// Data for the `stopped` recording event
#[derive(Clone, Debug, PartialEq, Eq)]
#[cfg_attr(feature = "serde", derive(Serialize, Deserialize))]
pub struct Stopped {
    /// The id of the recording that was stopped
    pub recording_id: RecordingId,
}

impl From<Stopped> for RecordingEvent {
    fn from(value: Stopped) -> Self {
        Self::Stopped(value)
    }
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

impl From<Error> for RecordingEvent {
    fn from(value: Error) -> Self {
        Self::Error(value)
    }
}
