// SPDX-FileCopyrightText: OpenTalk GmbH <mail@opentalk.eu>
//
// SPDX-License-Identifier: EUPL-1.2

//! Signaling events for the `recording` namespace

use opentalk_types_signaling_recording::{event::Error, StreamUpdated};

#[allow(unused_imports)]
use crate::imports::*;

/// Events sent out by the `recording` module
#[derive(Clone, Debug, PartialEq, Eq)]
#[cfg_attr(
    feature = "serde",
    derive(Serialize, Deserialize),
    serde(tag = "message", rename_all = "snake_case")
)]
pub enum RecordingEvent {
    /// Stream has an update
    StreamUpdated(StreamUpdated),

    /// An error happened when executing a `recording` command
    Error(Error),

    /// Indicates that the recorder was not started
    RecorderError(RecorderError),
}

impl From<StreamUpdated> for RecordingEvent {
    fn from(value: StreamUpdated) -> Self {
        Self::StreamUpdated(value)
    }
}

impl From<Error> for RecordingEvent {
    fn from(value: Error) -> Self {
        Self::Error(value)
    }
}

/// Recorder not started
#[derive(Clone, Debug, PartialEq, Eq)]
#[cfg_attr(
    feature = "serde",
    derive(Serialize, Deserialize),
    serde(tag = "error", rename_all = "snake_case")
)]
pub enum RecorderError {
    /// Indicates, that the recorder timed out when attempting to start
    Timeout,
}

impl From<RecorderError> for RecordingEvent {
    fn from(value: RecorderError) -> Self {
        Self::RecorderError(value)
    }
}
