// SPDX-FileCopyrightText: OpenTalk GmbH <mail@opentalk.eu>
//
// SPDX-License-Identifier: EUPL-1.2

//! Common types related to tariff information

use std::collections::{HashMap, HashSet};

use crate::core::TariffId;

#[allow(unused_imports)]
use crate::imports::*;

/// Information related to a specific tariff
#[derive(Clone, Debug, PartialEq, Eq)]
#[cfg_attr(feature = "serde", derive(Serialize, Deserialize))]
pub struct TariffResource {
    /// The ID of the tariff
    pub id: TariffId,

    /// The name of the tariff
    pub name: String,

    /// The quotas of the tariff
    pub quotas: HashMap<String, u32>,

    /// Enabled modules for the tariff (deprecated, use 'modules' instead)
    pub enabled_modules: HashSet<String>,

    /// Disabled features for the tariff  (deprecated, use 'modules' instead)
    pub disabled_features: HashSet<String>,

    /// Enabled modules for the tariff, including their enabled features
    pub modules: HashMap<String, TariffModuleResource>,
}

/// Tariff information related to a specific module
#[derive(Clone, Debug, PartialEq, Eq)]
#[cfg_attr(feature = "serde", derive(Serialize, Deserialize))]
pub struct TariffModuleResource {
    /// Enabled features for the tariff
    pub features: HashSet<String>,
}

#[cfg(test)]
mod test {
    use super::*;
    use pretty_assertions::assert_eq;
    use serde_json::json;

    #[test]
    fn tariff_resource() {
        let expected = json!({
            "id": "00000000-0000-0000-0000-000000000000",
            "name": "tariff name",
            "quotas": {
                "quota_a": 11,
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
            quotas: HashMap::from([("quota_a".to_owned(), 11u32)]),
            enabled_modules: HashSet::from(["mod_a".to_owned()]),
            disabled_features: HashSet::from(["feat_a".to_owned()]),
            modules: HashMap::from([(
                "mod_a".to_owned(),
                TariffModuleResource {
                    features: HashSet::from(["feat_a".to_owned()]),
                },
            )]),
        })
        .unwrap();

        assert_eq!(expected, produced);
    }
}
