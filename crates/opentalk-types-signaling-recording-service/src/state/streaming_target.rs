// SPDX-FileCopyrightText: OpenTalk GmbH <mail@opentalk.eu>
//
// SPDX-License-Identifier: EUPL-1.2

use url::Url;

use super::StreamStartOption;

/// The streaming target
#[derive(Debug, Clone, PartialEq, Eq)]
#[cfg_attr(feature = "serde", derive(serde::Deserialize, serde::Serialize))]
pub struct StreamingTarget {
    /// The start options for the target stream
    #[cfg_attr(feature = "serde", serde(flatten))]
    pub stream_start_options: StreamStartOption,

    /// The target Url to which the stream shall be streamed to
    #[cfg_attr(
        feature = "serde",
        serde(default, skip_serializing_if = "Option::is_none")
    )]
    pub location: Option<Url>,
}
