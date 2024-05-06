// SPDX-FileCopyrightText: OpenTalk GmbH <mail@opentalk.eu>
//
// SPDX-License-Identifier: EUPL-1.2

//! Signaling messages between runners

use opentalk_types::signaling::recording::StreamUpdated;
use serde::{Deserialize, Serialize};

/// The exchange message to send between runners
#[derive(Debug, Serialize, Deserialize)]
#[serde(rename_all = "snake_case", tag = "message", content = "content")]
pub enum Message {
    /// UpdateStream message
    StreamUpdated(StreamUpdated),

    /// Indicates that the recorder started.
    RecorderStarting,

    /// Indicates that the recorder is about to stop.
    RecorderStopping,
}
