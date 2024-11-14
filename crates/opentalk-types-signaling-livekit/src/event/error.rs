// SPDX-FileCopyrightText: OpenTalk GmbH <mail@opentalk.eu>
//
// SPDX-License-Identifier: EUPL-1.2

/// Livekit errors
#[derive(Clone, Debug, PartialEq, Eq)]
#[cfg_attr(
    feature = "serde",
    derive(serde::Serialize, serde::Deserialize),
    serde(tag = "error", rename_all = "snake_case")
)]
pub enum Error {
    /// Livekit server is not available
    LivekitUnavailable,
    /// Client has missing permissions to do the action on the livekit server
    InsufficientPermissions,
}
