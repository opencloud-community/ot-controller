// SPDX-FileCopyrightText: OpenTalk GmbH <mail@opentalk.eu>
//
// SPDX-License-Identifier: EUPL-1.2

use std::time::Duration;

/// Command to start a poll
#[derive(Clone, Debug, PartialEq, Eq)]
#[cfg_attr(feature = "serde", derive(serde::Serialize, serde::Deserialize))]
pub struct Start {
    /// The description of the poll topic
    pub topic: String,

    /// True if the poll is live
    #[cfg_attr(feature = "serde", serde(default))]
    pub live: bool,

    /// True if the poll accepts multiple choices
    #[cfg_attr(feature = "serde", serde(default))]
    pub multiple_choice: bool,

    /// The choices of the poll
    pub choices: Vec<String>,

    /// The duration of the poll
    #[cfg_attr(
        feature = "serde",
        serde(with = "opentalk_types_common::utils::duration_seconds")
    )]
    pub duration: Duration,
}
