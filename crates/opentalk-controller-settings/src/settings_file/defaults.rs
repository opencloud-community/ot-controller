// SPDX-FileCopyrightText: OpenTalk GmbH <mail@opentalk.eu>
//
// SPDX-License-Identifier: EUPL-1.2

use std::collections::BTreeSet;

use opentalk_types_common::{features::ModuleFeatureId, users::Language};
use serde::Deserialize;

#[derive(Clone, Default, Debug, PartialEq, Eq, Deserialize)]
pub struct Defaults {
    #[serde(default = "default_user_language")]
    pub user_language: Language,
    #[serde(default)]
    pub screen_share_requires_permission: bool,
    #[serde(default)]
    pub disabled_features: BTreeSet<ModuleFeatureId>,
}

fn default_user_language() -> Language {
    "en-US".parse().expect("valid language")
}
