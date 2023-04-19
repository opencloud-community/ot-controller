// SPDX-FileCopyrightText: OpenTalk GmbH <mail@opentalk.eu>
//
// SPDX-License-Identifier: EUPL-1.2

//! Signaling events for the `breakout` namespace

use crate::{
    core::{BreakoutRoomId, Timestamp},
    imports::*,
};

use super::BreakoutRoom;

/// Event signaling to the participant that the breakout session has started
#[derive(Clone, Debug, PartialEq, Eq)]
#[cfg_attr(feature = "serde", derive(Serialize, Deserialize))]
pub struct Started {
    /// List of the breakout rooms
    pub rooms: Vec<BreakoutRoom>,
    /// The expiration time of the breakout session
    pub expires: Option<Timestamp>,
    /// The id of the assigned breakout room
    pub assignment: Option<BreakoutRoomId>,
}

/// Error from the `breakout` module namespace
#[derive(Clone, Debug, PartialEq, Eq)]
#[cfg_attr(
    feature = "serde",
    derive(Serialize, Deserialize),
    serde(tag = "error", rename_all = "snake_case")
)]
pub enum Error {
    ///  No active breakout session is running
    Inactive,
    /// Insufficient permissions to perform a command
    InsufficientPermissions,
}
