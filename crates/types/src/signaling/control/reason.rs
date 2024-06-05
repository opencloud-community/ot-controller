// SPDX-FileCopyrightText: OpenTalk GmbH <mail@opentalk.eu>
//
// SPDX-License-Identifier: EUPL-1.2

#[allow(unused_imports)]
use crate::imports::*;

/// The reason for the Control::Left event
#[derive(Clone, Copy, Debug, PartialEq, Eq)]
#[cfg_attr(
    feature = "serde",
    derive(Serialize, Deserialize),
    serde(rename_all = "snake_case")
)]
pub enum Reason {
    /// The participant quit
    Quit,
    /// The websocket connection timed out
    Timeout,
    /// Sent to waiting room
    SentToWaitingRoom,
}
