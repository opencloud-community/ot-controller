// SPDX-FileCopyrightText: OpenTalk GmbH <mail@opentalk.eu>
//
// SPDX-License-Identifier: EUPL-1.2

use opentalk_types_signaling_recording::StreamStatus;

/// The recorder target
#[derive(Debug, Clone, PartialEq, Eq)]
#[cfg_attr(feature = "serde", derive(serde::Deserialize, serde::Serialize))]
pub struct StreamStartOption {
    /// Whether the stream shall be started automatically
    pub auto_connect: bool,

    /// The status of the stream
    pub status: StreamStatus,

    /// Whether the target stream shall be started as Paused
    pub start_paused: bool,
}
