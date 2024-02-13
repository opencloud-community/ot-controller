// SPDX-FileCopyrightText: OpenTalk GmbH <mail@opentalk.eu>
//
// SPDX-License-Identifier: EUPL-1.2

use serde::{Deserialize, Serialize};

use super::RecordingId;

#[derive(Debug, Serialize, Deserialize)]
pub enum Message {
    /// Signals for the recording "participant"
    Stop,

    /// Messages sent to participants to signal changes in the recording
    Started(RecordingId),
    Stopped(RecordingId),
}
