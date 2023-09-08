// SPDX-FileCopyrightText: OpenTalk GmbH <mail@opentalk.eu>
//
// SPDX-License-Identifier: EUPL-1.2

//! Signaling events for the `whiteboard` namespace

use crate::core::AssetId;

#[allow(unused_imports)]
use crate::imports::*;

use url::Url;

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

/// The access URL to a specific data
#[derive(Clone, Debug, PartialEq, Eq)]
#[cfg_attr(feature = "serde", derive(Serialize, Deserialize))]
pub struct AccessUrl {
    /// URL for the data
    pub url: Url,
}

impl From<AccessUrl> for WhiteboardEvent {
    fn from(value: AccessUrl) -> Self {
        Self::SpaceUrl(value)
    }
}

/// Handle to a PDF asset
#[derive(Clone, Debug, PartialEq, Eq)]
#[cfg_attr(feature = "serde", derive(Serialize, Deserialize))]
pub struct PdfAsset {
    /// The file name of the PDF asset
    pub filename: String,

    /// The asset id for the PDF asset
    pub asset_id: AssetId,
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
}

impl From<Error> for WhiteboardEvent {
    fn from(value: Error) -> Self {
        Self::Error(value)
    }
}
