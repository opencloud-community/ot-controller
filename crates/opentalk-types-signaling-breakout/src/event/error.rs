// SPDX-FileCopyrightText: OpenTalk GmbH <mail@opentalk.eu>
//
// SPDX-License-Identifier: EUPL-1.2

/// Error from the `breakout` module namespace
#[derive(Clone, Debug, PartialEq, Eq)]
#[cfg_attr(
    feature = "serde",
    derive(serde::Serialize, serde::Deserialize),
    serde(tag = "error", rename_all = "snake_case")
)]
pub enum Error {
    ///  No active breakout session is running
    Inactive,

    /// Insufficient permissions to perform a command
    InsufficientPermissions,
}
