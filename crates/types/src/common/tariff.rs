// SPDX-FileCopyrightText: OpenTalk GmbH <mail@opentalk.eu>
//
// SPDX-License-Identifier: EUPL-1.2

//! Common types related to tariff information

use std::collections::{BTreeMap, BTreeSet};

#[allow(unused_imports)]
use crate::imports::*;
use crate::{core::TariffId, utils::ExampleData};

/// Information related to a specific tariff
#[derive(Clone, Debug, PartialEq, Eq)]
#[cfg_attr(feature = "serde", derive(Serialize, Deserialize))]
#[cfg_attr(
    feature = "utoipa",
    derive(utoipa::ToSchema),
    schema(example = json!(TariffResource::example_data())),
)]
pub struct TariffResource {
    /// The ID of the tariff
    pub id: TariffId,

    /// The name of the tariff
    pub name: String,

    /// The quotas of the tariff
    pub quotas: BTreeMap<QuotaType, u64>,

    /// Enabled modules for the tariff (deprecated, use 'modules' instead)
    #[cfg_attr(feature = "utoipa", schema(deprecated))]
    pub enabled_modules: BTreeSet<String>,

    /// Disabled features for the tariff  (deprecated, use 'modules' instead)
    #[cfg_attr(feature = "utoipa", schema(deprecated))]
    pub disabled_features: BTreeSet<String>,

    /// Enabled modules for the tariff, including their enabled features
    pub modules: BTreeMap<String, TariffModuleResource>,
}

impl TariffResource {
    /// Query whether a specific feature for a module tariff is enabled
    pub fn has_feature_enabled(&self, module: &str, feature: &str) -> bool {
        self.modules
            .get(module)
            .map(|m| m.has_feature_enabled(feature))
            .unwrap_or_default()
    }

    /// Get the features for a module
    pub fn module_features(&self, module: &str) -> Option<&BTreeSet<String>> {
        self.modules.get(module).map(|m| &m.features)
    }
}

impl ExampleData for TariffResource {
    fn example_data() -> Self {
        Self {
            id: TariffId::nil(),
            name: "Starter tariff".to_string(),
            quotas: BTreeMap::from_iter([(QuotaType::MaxStorage, 50000)]),
            enabled_modules: BTreeSet::from_iter([
                "core".to_string(),
                "media".to_string(),
                "recording".to_string(),
                "chat".to_string(),
                "moderation".to_string(),
            ]),
            disabled_features: BTreeSet::from_iter(["recording::stream".to_string()]),
            modules: BTreeMap::from_iter([
                ("core".to_string(), TariffModuleResource::default()),
                ("media".to_string(), TariffModuleResource::default()),
                (
                    "recording".to_string(),
                    TariffModuleResource {
                        features: BTreeSet::from_iter(["record".to_string()]),
                    },
                ),
                ("chat".to_string(), TariffModuleResource::default()),
                ("moderation".to_string(), TariffModuleResource::default()),
            ]),
        }
    }
}

/// Tariff information related to a specific module
#[derive(Default, Clone, Debug, PartialEq, Eq)]
#[cfg_attr(feature = "serde", derive(Serialize, Deserialize))]
#[cfg_attr(feature = "utoipa", derive(utoipa::ToSchema))]
pub struct TariffModuleResource {
    /// Enabled features for the tariff
    pub features: BTreeSet<String>,
}

impl TariffModuleResource {
    /// Query whether a specific feature for a module tariff is enabled
    pub fn has_feature_enabled(&self, feature: &str) -> bool {
        self.features.contains(feature)
    }
}

/// The quota types that can be enforced on tenants.
#[derive(Debug, Clone, Ord, PartialOrd, Eq, PartialEq, Hash, strum::Display, strum::EnumString)]
#[strum(serialize_all = "snake_case")]
#[cfg_attr(
    feature = "serde",
    derive(Serialize, Deserialize),
    serde(rename_all = "snake_case")
)]
#[cfg_attr(
    feature = "clap",
    derive(clap::ValueEnum),
    clap(rename_all = "snake_case")
)]
pub enum QuotaType {
    /// This quota limits the total amount of data, measured bytes, that can be
    /// stored by the tenant. This is a soft limit which allows the user to store
    /// files as long as their usage is below the limit. Once the limit is reached
    /// or exceeded, no new data can be stored.
    MaxStorage,

    /// This quota restricts the total duration for which a tenant can utilize a
    /// meeting room, measured in seconds.
    RoomTimeLimitSecs,

    /// This quota sets a limit on the number of participants that can join a room.
    RoomParticipantLimit,

    /// Generic quota type.
    #[cfg_attr(feature = "serde", serde(untagged))]
    #[cfg_attr(feature = "clap", clap(skip))]
    #[strum(default)]
    Other(String),
}

#[cfg(test)]
mod test {
    use pretty_assertions::assert_eq;

    use super::*;

    #[cfg(feature = "serde")]
    #[test]
    fn quota_type_json() {
        use serde_json::json;
        let quota = BTreeMap::from([
            (QuotaType::MaxStorage, 11u64),
            (QuotaType::RoomTimeLimitSecs, 12u64),
            (QuotaType::RoomParticipantLimit, 13u64),
            (QuotaType::Other("this_is_somethingElse".to_string()), 14u64),
        ]);
        let quota_json_repr =
            serde_json::to_value(quota.clone()).expect("QuotaType must be serializable");

        assert_eq!(
            quota_json_repr,
            json!({
                "max_storage": 11,
                "room_time_limit_secs": 12,
                "room_participant_limit": 13,
                "this_is_somethingElse": 14
            })
        );
        assert_eq!(
            quota,
            serde_json::from_value(quota_json_repr).expect("Must be deserialize")
        );
    }

    #[cfg(feature = "clap")]
    #[test]
    fn quota_type_string() {
        use std::str::FromStr;

        assert_eq!(
            QuotaType::from_str("max_storage").unwrap(),
            QuotaType::MaxStorage
        );
        assert_eq!(
            QuotaType::from_str("room_time_limit_secs").unwrap(),
            QuotaType::RoomTimeLimitSecs
        );
        assert_eq!(
            QuotaType::from_str("room_participant_limit").unwrap(),
            QuotaType::RoomParticipantLimit
        );
        assert_eq!(
            QuotaType::from_str("this_is_somethingElse").unwrap(),
            QuotaType::Other("this_is_somethingElse".to_string())
        );
    }

    #[cfg(feature = "serde")]
    #[test]
    fn tariff_resource() {
        use serde_json::json;
        let expected = json!({
            "id": "00000000-0000-0000-0000-000000000000",
            "name": "tariff name",
            "quotas": {
                "max_storage": 11,
                "room_time_limit_secs": 12,
                "room_participant_limit": 13,
                "this_is_somethingElse": 14,
            },
            "enabled_modules": ["mod_a"],
            "disabled_features": ["feat_a"],
            "modules": {
                "mod_a": {
                    "features": ["feat_a"],
                },
            },
        });

        let produced = serde_json::to_value(TariffResource {
            id: TariffId::nil(),
            name: "tariff name".to_string(),
            quotas: BTreeMap::from([
                (QuotaType::MaxStorage, 11u64),
                (QuotaType::RoomTimeLimitSecs, 12u64),
                (QuotaType::RoomParticipantLimit, 13u64),
                (QuotaType::Other("this_is_somethingElse".to_string()), 14u64),
            ]),
            enabled_modules: BTreeSet::from(["mod_a".to_owned()]),
            disabled_features: BTreeSet::from(["feat_a".to_owned()]),
            modules: BTreeMap::from([(
                "mod_a".to_owned(),
                TariffModuleResource {
                    features: BTreeSet::from(["feat_a".to_owned()]),
                },
            )]),
        })
        .unwrap();

        assert_eq!(expected, produced);
    }
}
