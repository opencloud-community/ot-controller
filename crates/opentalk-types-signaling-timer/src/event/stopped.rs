// SPDX-FileCopyrightText: OpenTalk GmbH <mail@opentalk.eu>
//
// SPDX-License-Identifier: EUPL-1.2

use super::StopKind;
use crate::TimerId;

/// The current timer has been stopped
#[derive(Debug, PartialEq, Eq)]
#[cfg_attr(feature = "serde", derive(serde::Serialize, serde::Deserialize))]
pub struct Stopped {
    /// The timer id
    pub timer_id: TimerId,
    /// The stop kind
    #[cfg_attr(feature = "serde", serde(flatten))]
    pub kind: StopKind,
    /// An optional reason to all participants. Set by moderator
    #[cfg_attr(feature = "serde", serde(skip_serializing_if = "Option::is_none"))]
    pub reason: Option<String>,
}
