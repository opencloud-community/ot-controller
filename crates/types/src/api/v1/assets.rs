// SPDX-FileCopyrightText: OpenTalk GmbH <mail@opentalk.eu>
//
// SPDX-License-Identifier: EUPL-1.2

//! This module contains types that are used for OpenTalk API V1 assets endpoints.

use chrono::{DateTime, Utc};

use crate::core::AssetId;
#[allow(unused_imports)]
use crate::imports::*;

/// Representation of an asset resource
#[derive(Clone, Debug, PartialEq, Eq)]
#[cfg_attr(feature = "serde", derive(Serialize, Deserialize))]
pub struct AssetResource {
    /// The ID of an asset
    pub id: AssetId,

    /// The filename of the asset
    pub filename: String,

    /// The namespace of the asset
    #[cfg_attr(feature = "serde", serde(skip_serializing_if = "Option::is_none"))]
    pub namespace: Option<String>,

    /// The timestamp the asset was created
    pub created_at: DateTime<Utc>,

    /// The asset kind
    pub kind: String,

    /// The size of the asset in bytes
    pub size: i64,
}
