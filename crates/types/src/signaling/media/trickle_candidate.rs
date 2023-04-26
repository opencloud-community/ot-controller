// SPDX-FileCopyrightText: OpenTalk GmbH <mail@opentalk.eu>
//
// SPDX-License-Identifier: EUPL-1.2

#[allow(unused_imports)]
use crate::imports::*;

/// A candidate for ICE/SDP trickle
#[derive(Clone, Debug, PartialEq, Eq)]
#[cfg_attr(feature = "serde", derive(Serialize, Deserialize))]
pub struct TrickleCandidate {
    /// The SDP m-line index
    #[cfg_attr(feature = "serde", serde(rename = "sdpMLineIndex"))]
    pub sdp_m_line_index: u64,

    /// The ICE candidate string
    pub candidate: String,
}
