// SPDX-FileCopyrightText: OpenTalk GmbH <mail@opentalk.eu>
//
// SPDX-License-Identifier: EUPL-1.2

use crate::TimerId;

/// Update the ready status
#[derive(Debug)]
#[cfg_attr(feature = "serde", derive(serde::Serialize, serde::Deserialize))]
pub struct UpdateReadyStatus {
    /// The timer id
    pub timer_id: TimerId,
    /// The new status
    pub status: bool,
}
