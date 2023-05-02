// SPDX-FileCopyrightText: OpenTalk GmbH <mail@opentalk.eu>
//
// SPDX-License-Identifier: EUPL-1.2

//! Types related to signaling events in the `protocol` namespace

use crate::{core::AssetId, imports::*};

/// Handle to a PDF asset
#[derive(Clone, Debug, PartialEq, Eq)]
#[cfg_attr(feature = "serde", derive(Serialize, Deserialize))]
pub struct PdfAsset {
    /// The file name of the PDF asset
    pub filename: String,

    /// The asset id for the PDF asset
    pub asset_id: AssetId,
}

/// Errors from the `protocol` module namespace
#[derive(Clone, Debug, PartialEq, Eq)]
#[cfg_attr(
    feature = "serde",
    derive(Serialize, Deserialize),
    serde(rename_all = "snake_case", tag = "error")
)]
pub enum Error {
    /// The requesting user has insufficient permissions for the operation
    InsufficientPermissions,
    /// The request contains invalid participant ids
    InvalidParticipantSelection,
    /// Is send when another instance just started initializing and etherpad is not available yet
    CurrentlyInitializing,
    /// The etherpad initialization failed
    FailedInitialization,
    /// The etherpad is not yet initailized
    NotInitialized,
}
