// SPDX-FileCopyrightText: OpenTalk GmbH <mail@opentalk.eu>
//
// SPDX-License-Identifier: EUPL-1.2

//! Peer frontend data for `recording` namespace

use crate::imports::*;

/// The state of other participants in the `recording` module.
///
/// This struct is sent to the participant in the `join_success` message
/// which will contain this information for each participant in the meeting.
#[derive(Clone, Debug, PartialEq, Eq)]
#[cfg_attr(feature = "serde", derive(Serialize, Deserialize))]
pub struct RecordingPeerState {
    /// Flag showing whether the participant consents to recording
    pub consents_recording: bool,
}
