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

    /// Enabled modules for the tariff
    pub enabled_modules: HashSet<String>,

    /// Disabled features for the tariff
    pub disabled_features: HashSet<String>,
}
