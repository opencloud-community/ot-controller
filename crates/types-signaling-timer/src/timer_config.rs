// SPDX-FileCopyrightText: OpenTalk GmbH <mail@opentalk.eu>
//
// SPDX-License-Identifier: EUPL-1.2

use opentalk_types_common::time::Timestamp;

use crate::{Kind, TimerId};

/// Status of a currently active timer
#[derive(Debug, PartialEq, Eq)]
#[cfg_attr(feature = "serde", derive(serde::Serialize, serde::Deserialize))]
pub struct TimerConfig {
    /// The timer id
    pub timer_id: TimerId,
    /// start time of the timer
    pub started_at: Timestamp,
    /// Timer kind
    #[cfg_attr(feature = "serde", serde(flatten))]
    pub kind: Kind,
    /// Style to use for the timer. Set by the sender.
    #[cfg_attr(feature = "serde", serde(skip_serializing_if = "Option::is_none"))]
    pub style: Option<String>,
    /// The optional title of the timer
    #[cfg_attr(feature = "serde", serde(skip_serializing_if = "Option::is_none"))]
    pub title: Option<String>,
    /// Flag to allow/disallow participants to mark themselves as ready
    pub ready_check_enabled: bool,
}
