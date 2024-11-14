// SPDX-FileCopyrightText: OpenTalk GmbH <mail@opentalk.eu>
//
// SPDX-License-Identifier: EUPL-1.2

use crate::StreamErrorReason;

/// The current status of a stream
#[derive(Debug, Clone, PartialEq, Eq, PartialOrd, Ord)]
#[cfg_attr(
    feature = "serde",
    derive(serde::Serialize, serde::Deserialize),
    serde(rename_all = "snake_case", tag = "status")
)]
pub enum StreamStatus {
    /// The stream is starting (meaning, that the Recorder is currently starting but is not yet confirmed to be started).
    Starting,

    /// The stream is inactive
    Inactive,

    /// The stream is active
    Active,

    /// The stream is paused
    Paused,

    /// The stream has returned an error
    Error {
        /// The error reason
        reason: StreamErrorReason,
    },
}
