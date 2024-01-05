// SPDX-FileCopyrightText: OpenTalk GmbH <mail@opentalk.eu>
//
// SPDX-License-Identifier: EUPL-1.2

//! Peer frontend data for `protocol` namespace

#[allow(unused_imports)]
use crate::imports::*;

/// The state of other participants in the `recording` module.
///
/// This struct is sent to the participant in the `join_success` message
/// which will contain this information for each participant in the meeting.
#[derive(Clone, Debug, PartialEq, Eq)]
#[cfg_attr(
    feature = "serde",
    derive(Serialize, Deserialize),
    serde(rename_all = "snake_case")
)]
pub struct ProtocolPeerState {
    /// Read-only access
    pub readonly: bool,
}

#[cfg(feature = "serde")]
impl SignalingModulePeerFrontendData for ProtocolPeerState {
    const NAMESPACE: Option<&'static str> = Some(super::NAMESPACE);
}
