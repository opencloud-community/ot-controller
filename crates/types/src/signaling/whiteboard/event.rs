// SPDX-FileCopyrightText: OpenTalk GmbH <mail@opentalk.eu>
//
// SPDX-License-Identifier: EUPL-1.2

//! Signaling events for the `whiteboard` namespace

use opentalk_types_signaling_whiteboard::event::{AccessUrl, PdfAsset};

#[allow(unused_imports)]
use crate::imports::*;

/// Events sent out by the `whiteboard` module
#[derive(Clone, Debug, PartialEq, Eq)]
#[cfg_attr(
    feature = "serde",
    derive(Serialize, Deserialize),
    serde(rename_all = "snake_case", tag = "message")
)]
pub enum WhiteboardEvent {
    /// A Spacedeck instance has been initialized
    SpaceUrl(AccessUrl),

    /// A PDF asset has been created
    PdfAsset(PdfAsset),

    /// An error happened when executing a `whiteboard` command
    Error(Error),
}

impl From<AccessUrl> for WhiteboardEvent {
    fn from(value: AccessUrl) -> Self {
        Self::SpaceUrl(value)
    }
}

impl From<PdfAsset> for WhiteboardEvent {
    fn from(value: PdfAsset) -> Self {
        Self::PdfAsset(value)
    }
}

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
    /// The requesting user has exceeded their storage
    StorageExceeded,
}

impl From<Error> for WhiteboardEvent {
    fn from(value: Error) -> Self {
        Self::Error(value)
    }
}
