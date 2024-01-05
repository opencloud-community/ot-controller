// SPDX-FileCopyrightText: OpenTalk GmbH <mail@opentalk.eu>
//
// SPDX-License-Identifier: EUPL-1.2

//! Frontend data for `breakout` namespace

use crate::core::{BreakoutRoomId, Timestamp};

#[allow(unused_imports)]
use crate::imports::*;

use super::{BreakoutRoom, ParticipantInOtherRoom};

/// The state the `breakout` module.
///
/// This struct is sent to the participant in the `join_success` message
/// when they join successfully to the meeting.
#[derive(Clone, Debug, PartialEq, Eq)]
#[cfg_attr(feature = "serde", derive(Serialize, Deserialize))]
pub struct BreakoutState {
    /// The id of the current breakout room
    pub current: Option<BreakoutRoomId>,

    /// The expiration timestamp for the breakout session
    pub expires: Option<Timestamp>,

    /// The breakout rooms in the breakout session
    pub rooms: Vec<BreakoutRoom>,

    /// The participants in the other breakout rooms
    pub participants: Vec<ParticipantInOtherRoom>,
}

#[cfg(feature = "serde")]
impl SignalingModuleFrontendData for BreakoutState {
    const NAMESPACE: Option<&'static str> = Some(super::NAMESPACE);
}
