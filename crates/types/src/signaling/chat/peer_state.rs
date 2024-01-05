// SPDX-FileCopyrightText: OpenTalk GmbH <mail@opentalk.eu>
//
// SPDX-License-Identifier: EUPL-1.2

//! Peer frontend data for `chat` namespace

use crate::core::GroupName;

#[allow(unused_imports)]
use crate::imports::*;

/// The state of other participants in the `chat` module.
///
/// This struct is sent to the participant in the `join_success` message
/// which will contain this information for each participant in the meeting.
#[derive(Clone, Debug)]
#[cfg_attr(feature = "serde", derive(Serialize, Deserialize))]
pub struct ChatPeerState {
    /// A list of group chats
    pub groups: Vec<GroupName>,
}

#[cfg(feature = "serde")]
impl SignalingModulePeerFrontendData for ChatPeerState {
    const NAMESPACE: Option<&'static str> = Some(super::NAMESPACE);
}
