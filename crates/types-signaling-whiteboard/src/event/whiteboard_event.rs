// SPDX-FileCopyrightText: OpenTalk GmbH <mail@opentalk.eu>
//
// SPDX-License-Identifier: EUPL-1.2

//! Signaling events for the `whiteboard` namespace

use crate::event::{AccessUrl, Error, PdfAsset};

/// Events sent out by the `whiteboard` module
#[derive(Clone, Debug, PartialEq, Eq)]
#[cfg_attr(
    feature = "serde",
    derive(serde::Serialize, serde::Deserialize),
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

impl From<Error> for WhiteboardEvent {
    fn from(value: Error) -> Self {
        Self::Error(value)
    }
}
