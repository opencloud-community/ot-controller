// SPDX-FileCopyrightText: OpenTalk GmbH <mail@opentalk.eu>
//
// SPDX-License-Identifier: EUPL-1.2

use super::Kind;

/// Start a new timer
#[derive(Debug)]
#[cfg_attr(feature = "serde", derive(serde::Serialize, serde::Deserialize))]
pub struct Start {
    /// The timer kind
    #[cfg_attr(feature = "serde", serde(flatten))]
    pub kind: Kind,
    /// An optional string tag to flag this timer with a custom style
    pub style: Option<String>,
    /// An optional title for the timer
    pub title: Option<String>,
    /// Flag to allow/disallow participants to mark themselves as ready
    #[cfg_attr(feature = "serde", serde(default))]
    pub enable_ready_check: bool,
}
