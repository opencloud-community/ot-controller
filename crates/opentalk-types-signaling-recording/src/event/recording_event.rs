// SPDX-FileCopyrightText: OpenTalk GmbH <mail@opentalk.eu>
//
// SPDX-License-Identifier: EUPL-1.2

use crate::{
    event::{Error, RecorderError},
    StreamUpdated,
};

/// Events sent out by the `recording` module
#[derive(Clone, Debug, PartialEq, Eq)]
#[cfg_attr(
    feature = "serde",
    derive(serde::Serialize, serde::Deserialize),
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

impl From<RecorderError> for RecordingEvent {
    fn from(value: RecorderError) -> Self {
        Self::RecorderError(value)
    }
}
