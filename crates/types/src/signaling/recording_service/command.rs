// SPDX-FileCopyrightText: OpenTalk GmbH <mail@opentalk.eu>
//
// SPDX-License-Identifier: EUPL-1.2

//! Signaling events for the `recording_service` namespace

use std::collections::BTreeSet;

use crate::core::StreamingTargetId;
#[allow(unused_imports)]
use crate::imports::*;

/// Events sent out by the `recording_service` module to the recorder
#[derive(Clone, Debug, PartialEq, Eq)]
#[cfg_attr(
    feature = "serde",
    derive(Serialize, Deserialize),
    serde(tag = "action", rename_all = "snake_case")
)]
pub enum RecordingServiceCommand {
    /// Start Streams
    StartStreams {
        /// The ids of the streams that should be started.
        target_ids: BTreeSet<StreamingTargetId>,
    },
    /// Pause Streams
    PauseStreams {
        /// The ids of the streams that should be paused.
        target_ids: BTreeSet<StreamingTargetId>,
    },
    /// Stop Streams
    StopStreams {
        /// The ids of the streams that should be stopped.
        target_ids: BTreeSet<StreamingTargetId>,
    },
}

/// Error from the `recording_service` module namespace
#[derive(Clone, Debug, PartialEq, Eq)]
#[cfg_attr(
    feature = "serde",
    derive(Serialize, Deserialize),
    serde(tag = "error", rename_all = "snake_case")
)]
pub enum Error {
    /// The participant has insufficient permissions to perform a command
    InsufficientPermissions,

    /// Invalid streaming id used
    InvalidStreamingId,
}
