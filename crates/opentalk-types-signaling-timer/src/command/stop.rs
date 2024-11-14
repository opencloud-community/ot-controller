// SPDX-FileCopyrightText: OpenTalk GmbH <mail@opentalk.eu>
//
// SPDX-License-Identifier: EUPL-1.2

use crate::TimerId;

/// Stop a running timer
#[derive(Debug)]
#[cfg_attr(feature = "serde", derive(serde::Serialize, serde::Deserialize))]
pub struct Stop {
    /// The timer id
    pub timer_id: TimerId,
    /// An optional reason for the stop
    pub reason: Option<String>,
}
