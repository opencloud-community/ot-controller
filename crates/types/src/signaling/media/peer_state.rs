// SPDX-FileCopyrightText: OpenTalk GmbH <mail@opentalk.eu>
//
// SPDX-License-Identifier: EUPL-1.2

//! Peer frontend data for `recording` namespace

#[allow(unused_imports)]
use crate::imports::*;

use super::ParticipantMediaState;

/// The state of other participants in the `recording` module.
///
/// This struct is sent to the participant in the `join_success` message
/// which will contain this information for each participant in the meeting.
#[derive(Clone, Debug, Default, PartialEq, Eq)]
#[cfg_attr(feature = "serde", derive(Serialize, Deserialize))]
pub struct MediaPeerState {
    /// The media state of the peer
    #[cfg_attr(feature = "serde", serde(flatten))]
    pub state: Option<ParticipantMediaState>,

    /// Whether the participant has permission to share the screen
    pub is_presenter: bool,
}

#[cfg(feature = "serde")]
impl SignalingModulePeerFrontendData for MediaPeerState {
    const NAMESPACE: Option<&'static str> = Some(super::NAMESPACE);
}
