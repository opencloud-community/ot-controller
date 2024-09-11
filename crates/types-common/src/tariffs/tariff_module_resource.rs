// SPDX-FileCopyrightText: OpenTalk GmbH <mail@opentalk.eu>
//
// SPDX-License-Identifier: EUPL-1.2

use std::collections::BTreeSet;

use crate::features::FeatureId;
#[allow(unused_imports)]
use crate::imports::*;

/// Tariff information related to a specific module
#[derive(Default, Clone, Debug, PartialEq, Eq)]
#[cfg_attr(feature = "serde", derive(Serialize, Deserialize))]
#[cfg_attr(feature = "utoipa", derive(utoipa::ToSchema))]
pub struct TariffModuleResource {
    /// Enabled features for the tariff
    pub features: BTreeSet<FeatureId>,
}

impl TariffModuleResource {
    /// Query whether a specific feature for a module tariff is enabled
    pub fn has_feature_enabled(&self, feature: &FeatureId) -> bool {
        self.features.contains(feature)
    }
}
