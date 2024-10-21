// SPDX-FileCopyrightText: OpenTalk GmbH <mail@opentalk.eu>
//
// SPDX-License-Identifier: EUPL-1.2

//! Signaling commands for the `recording` namespace

use opentalk_types_signaling_recording::StreamUpdated;

/// Commands for the `recording_service` namespace
#[derive(Clone, Debug, PartialEq, Eq)]
#[cfg_attr(
    feature = "serde",
    derive(serde::Serialize, serde::Deserialize),
    serde(tag = "message", rename_all = "snake_case")
)]
pub enum RecordingServiceEvent {
    /// Stream has an update
    StreamUpdated(StreamUpdated),
}
