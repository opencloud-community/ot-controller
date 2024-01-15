// SPDX-FileCopyrightText: OpenTalk GmbH <mail@opentalk.eu>
//
// SPDX-License-Identifier: EUPL-1.2

use super::RecordingId;
use serde::{Deserialize, Serialize};

#[derive(Debug, Serialize, Deserialize)]
pub enum Message {
    /// Signals for the recording "participant"
    Stop,

    /// Messages sent to participants to signal changes in the recording
    Started(RecordingId),
    Stopped(RecordingId),
}
