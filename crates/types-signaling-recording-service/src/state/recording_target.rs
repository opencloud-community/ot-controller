// SPDX-FileCopyrightText: OpenTalk GmbH <mail@opentalk.eu>
//
// SPDX-License-Identifier: EUPL-1.2

use super::StreamStartOption;

/// The recorder target
#[derive(Debug, Clone, PartialEq, Eq)]
#[cfg_attr(feature = "serde", derive(serde::Deserialize, serde::Serialize))]
pub struct RecordingTarget {
    /// The start options for the target stream
    #[cfg_attr(feature = "serde", serde(flatten))]
    pub stream_start_options: StreamStartOption,
}
