// SPDX-FileCopyrightText: OpenTalk GmbH <mail@opentalk.eu>
//
// SPDX-License-Identifier: EUPL-1.2

//! Signaling events for the `chat` namespace

#[allow(unused_imports)]
use crate::imports::*;

/// Errors from the `chat` module namespace
#[derive(Debug, Clone, PartialEq, Eq)]
#[cfg_attr(
    feature = "serde",
    derive(Serialize, Deserialize),
    serde(tag = "error", rename_all = "snake_case")
)]
pub enum Error {
    /// Request while chat is disabled
    ChatDisabled,

    /// Request user has insufficient permissions
    InsufficientPermissions,
}
