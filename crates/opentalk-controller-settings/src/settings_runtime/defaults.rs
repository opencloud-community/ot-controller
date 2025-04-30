// SPDX-FileCopyrightText: OpenTalk GmbH <mail@opentalk.eu>
//
// SPDX-License-Identifier: EUPL-1.2

use std::collections::BTreeSet;

use opentalk_types_common::{features::ModuleFeatureId, users::Language};

use crate::settings_file;

/// Some settings that apply for the whole installation.
#[derive(Debug, Clone, PartialEq, Eq)]
pub struct Defaults {
    /// The user language.
    pub user_language: Language,

    /// Flag indicating whether the screen share requires an explicit permission.
    pub screen_share_requires_permission: bool,

    /// A list of disabled features.
    pub disabled_features: BTreeSet<ModuleFeatureId>,
}

impl From<settings_file::Defaults> for Defaults {
    fn from(
        settings_file::Defaults {
            user_language,
            screen_share_requires_permission,
            disabled_features,
        }: settings_file::Defaults,
    ) -> Self {
        Self {
            user_language: user_language.unwrap_or_else(default_user_language),
            screen_share_requires_permission: screen_share_requires_permission.unwrap_or_default(),
            disabled_features,
        }
    }
}

impl Default for Defaults {
    fn default() -> Self {
        Self {
            user_language: default_user_language(),
            screen_share_requires_permission: false,
            disabled_features: BTreeSet::default(),
        }
    }
}

pub(crate) fn default_user_language() -> Language {
    "en-US".parse().expect("valid language")
}
