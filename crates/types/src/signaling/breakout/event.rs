// SPDX-FileCopyrightText: OpenTalk GmbH <mail@opentalk.eu>
//
// SPDX-License-Identifier: EUPL-1.2

//! Signaling events for the `breakout` namespace

use crate::imports::*;

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
