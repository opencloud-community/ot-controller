// SPDX-FileCopyrightText: OpenTalk GmbH <mail@opentalk.eu>
//
// SPDX-License-Identifier: EUPL-1.2

use crate::TimerConfig;

/// A timer has been started
#[derive(Debug, PartialEq, Eq)]
#[cfg_attr(feature = "serde", derive(serde::Serialize, serde::Deserialize))]
pub struct Started {
    /// Config of the started timer
    #[cfg_attr(feature = "serde", serde(flatten))]
    pub config: TimerConfig,
}
