// SPDX-FileCopyrightText: OpenTalk GmbH <mail@opentalk.eu>
//
// SPDX-License-Identifier: EUPL-1.2

//! Common types related to tariff information

use std::collections::{BTreeMap, BTreeSet};

use crate::{
    features::FeatureId,
    modules::ModuleId,
    tariffs::{QuotaType, TariffId, TariffModuleResource},
    utils::ExampleData,
};

/// Information related to a specific tariff
#[derive(Clone, Debug, PartialEq, Eq)]
#[cfg_attr(feature = "serde", derive(serde::Serialize, serde::Deserialize))]
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

impl ExampleData for TariffResource {
    fn example_data() -> Self {
        Self {
            id: TariffId::nil(),
            name: "Starter tariff".to_string(),
            quotas: BTreeMap::from_iter([(QuotaType::MaxStorage, 50000)]),
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

#[cfg(all(test, feature = "serde"))]
mod serde_tests {
    use pretty_assertions::assert_eq;

    use super::*;

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
