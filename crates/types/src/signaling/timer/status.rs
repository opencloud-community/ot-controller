// SPDX-FileCopyrightText: OpenTalk GmbH <mail@opentalk.eu>
//
// SPDX-License-Identifier: EUPL-1.2

//! Status data for `timer` namespace

#[allow(unused_imports)]
use crate::imports::*;

use super::TimerConfig;

/// Status of and belonging to a currently active timer
#[derive(Debug, PartialEq, Eq)]
#[cfg_attr(feature = "serde", derive(Serialize, Deserialize))]
pub struct TimerStatus {
    /// config of the timer
    #[cfg_attr(feature = "serde", serde(flatten))]
    pub config: TimerConfig,

    /// Flag to indicate that the current participant has marked themselves as ready
    #[cfg_attr(feature = "serde", serde(skip_serializing_if = "Option::is_none"))]
    pub ready_status: Option<bool>,
}

#[cfg(feature = "serde")]
impl SignalingModuleFrontendData for TimerStatus {
    const NAMESPACE: Option<&'static str> = Some(super::NAMESPACE);
}
