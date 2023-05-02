// SPDX-FileCopyrightText: OpenTalk GmbH <mail@opentalk.eu>
//
// SPDX-License-Identifier: EUPL-1.2

//! Signaling events for the `whiteboard` namespace

use crate::imports::*;

/// Error from the `whiteboard` module namespace
#[derive(Clone, Debug, PartialEq, Eq)]
#[cfg_attr(
    feature = "serde",
    derive(Serialize, Deserialize),
    serde(tag = "error", rename_all = "snake_case")
)]
pub enum Error {
    /// The requesting user has insufficient permissions for the operation
    InsufficientPermissions,
    /// Is sent when another instance is currently initializing spacedeck
    CurrentlyInitializing,
    /// The spacedeck initialization failed
    InitializationFailed,
    /// Spacedeck is already initialized
    AlreadyInitialized,
}
