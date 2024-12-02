// SPDX-FileCopyrightText: OpenTalk GmbH <mail@opentalk.eu>
//
// SPDX-License-Identifier: EUPL-1.2

use chrono::{DateTime, TimeZone as _, Utc};
use opentalk_types_common::{assets::AssetId, modules::ModuleId, utils::ExampleData};

/// Representation of an asset resource
#[derive(Clone, Debug, PartialEq, Eq)]
#[cfg_attr(feature = "serde", derive(serde::Serialize, serde::Deserialize))]
#[cfg_attr(
    feature="utoipa",
    derive(utoipa::ToSchema),
    schema(example = json!(AssetResource::example_data())),
)]
pub struct AssetResource {
    /// The ID of an asset
    pub id: AssetId,

    /// The filename of the asset
    pub filename: String,

    /// The namespace of the asset
    // Field is non-required already, utoipa adds a `nullable: true` entry
    // by default which creates a false positive in the spectral linter when
    // combined with example data.
    #[cfg_attr(feature = "utoipa", schema(nullable = false))]
    #[cfg_attr(feature = "serde", serde(skip_serializing_if = "Option::is_none"))]
    pub namespace: Option<ModuleId>,

    /// The timestamp the asset was created
    pub created_at: DateTime<Utc>,

    /// The asset kind
    pub kind: String,

    /// The size of the asset in bytes
    pub size: i64,
}

impl ExampleData for AssetResource {
    fn example_data() -> Self {
        Self {
            id: AssetId::example_data(),
            filename: "recording.webm".to_string(),
            namespace: Some("recording".parse().expect("valid module id")),
            created_at: Utc.with_ymd_and_hms(2024, 6, 18, 11, 22, 33).unwrap(),
            kind: "record".to_string(),
            size: 98765432,
        }
    }
}
