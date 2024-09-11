// SPDX-FileCopyrightText: OpenTalk GmbH <mail@opentalk.eu>
//
// SPDX-License-Identifier: EUPL-1.2

//! Common types related to tariff information

use std::collections::{BTreeMap, BTreeSet};

#[allow(unused_imports)]
use crate::imports::*;
use crate::{
    features::{FeatureId, ModuleFeatureId},
    modules::ModuleId,
    tariffs::{QuotaType, TariffId, TariffModuleResource},
    utils::ExampleData,
};

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
    pub enabled_modules: BTreeSet<ModuleId>,

    /// Disabled features for the tariff  (deprecated, use 'modules' instead)
    #[cfg_attr(feature = "utoipa", schema(deprecated))]
    #[cfg_attr(
        feature = "serde",
        serde(serialize_with = "serialize_module_disabled_features")
    )]
    pub disabled_features: BTreeSet<ModuleFeatureId>,

    /// Enabled modules for the tariff, including their enabled features
    pub modules: BTreeMap<ModuleId, TariffModuleResource>,
}

impl TariffResource {
    /// Query whether a specific feature for a module tariff is enabled
    pub fn has_feature_enabled(&self, module: &ModuleId, feature: &FeatureId) -> bool {
        self.modules
            .get(module)
            .map(|m| m.has_feature_enabled(feature))
            .unwrap_or_default()
    }

    /// Get the features for a module
    pub fn module_features(&self, module: &ModuleId) -> Option<&BTreeSet<FeatureId>> {
        self.modules.get(module).map(|m| &m.features)
    }
}

#[cfg(feature = "serde")]
fn serialize_module_disabled_features<S: Serializer>(
    value: &BTreeSet<ModuleFeatureId>,
    serializer: S,
) -> Result<S::Ok, S::Error> {
    let v = value
        .iter()
        .flat_map(|v| {
            if v.module.is_default() {
                vec![v.to_string(), v.feature.to_string()]
            } else {
                vec![v.to_string()]
            }
        })
        .collect::<Vec<_>>();
    v.serialize(serializer)
}

impl ExampleData for TariffResource {
    fn example_data() -> Self {
        Self {
            id: TariffId::nil(),
            name: "Starter tariff".to_string(),
            quotas: BTreeMap::from_iter([(QuotaType::MaxStorage, 50000)]),
            enabled_modules: ["core", "media", "recording", "chat", "moderation"]
                .into_iter()
                .map(|s| s.parse::<ModuleId>())
                .collect::<Result<BTreeSet<ModuleId>, _>>()
                .expect("valid module ids"),
            disabled_features: ["recording::stream"]
                .into_iter()
                .map(|s| s.parse::<ModuleFeatureId>())
                .collect::<Result<BTreeSet<ModuleFeatureId>, _>>()
                .expect("valid module feature ids"),
            modules: [
                ("core", TariffModuleResource::default()),
                ("media", TariffModuleResource::default()),
                (
                    "recording",
                    TariffModuleResource {
                        features: BTreeSet::from_iter(["record"
                            .parse()
                            .expect("valid feature id")]),
                    },
                ),
                ("chat", TariffModuleResource::default()),
                ("moderation", TariffModuleResource::default()),
            ]
            .into_iter()
            .map(|(module, resource)| {
                (
                    module.parse::<ModuleId>().expect("valid module id"),
                    resource,
                )
            })
            .collect(),
        }
    }
}

#[cfg(test)]
mod test {
    use pretty_assertions::assert_eq;

    use super::*;

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
            "disabled_features": ["mod_a::feat_a"],
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
            enabled_modules: ["mod_a"]
                .into_iter()
                .map(|s| s.parse::<ModuleId>())
                .collect::<Result<BTreeSet<ModuleId>, _>>()
                .expect("valid module ids"),
            disabled_features: ["mod_a::feat_a"]
                .into_iter()
                .map(|s| s.parse::<ModuleFeatureId>())
                .collect::<Result<BTreeSet<ModuleFeatureId>, _>>()
                .expect("valid module feature ids"),
            modules: [(
                "mod_a",
                TariffModuleResource {
                    features: BTreeSet::from(["feat_a".parse().expect("valid feature id")]),
                },
            )]
            .into_iter()
            .map(|(module, resource)| {
                (
                    module.parse::<ModuleId>().expect("valid module id"),
                    resource,
                )
            })
            .collect(),
        })
        .unwrap();

        assert_eq!(expected, produced);
    }
}
