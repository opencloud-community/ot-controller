// SPDX-FileCopyrightText: OpenTalk GmbH <mail@opentalk.eu>
//
// SPDX-License-Identifier: EUPL-1.2

//! Frontend data for `recording` namespace

#[allow(unused_imports)]
use crate::imports::*;

use super::RecordingStatus;

/// The state of the `recording` module.
///
/// This struct is sent to the participant in the `join_success` message
/// when they join successfully to the meeting.
#[derive(Clone, Debug, PartialEq, Eq)]
#[cfg_attr(feature = "serde", derive(Serialize, Deserialize))]
pub struct RecordingState(pub Option<RecordingStatus>);

#[cfg(feature = "serde")]
impl SignalingModuleFrontendData for RecordingState {
    const NAMESPACE: Option<&'static str> = Some(super::NAMESPACE);
}
