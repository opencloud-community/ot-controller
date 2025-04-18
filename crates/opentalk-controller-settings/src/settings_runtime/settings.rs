// SPDX-FileCopyrightText: OpenTalk GmbH <mail@opentalk.eu>
//
// SPDX-License-Identifier: EUPL-1.2

use std::sync::Arc;

use crate::{Result, SettingsError, SettingsRaw};

#[derive(Debug, Clone, PartialEq, Eq)]
pub struct Settings {
    pub settings_raw: Arc<SettingsRaw>,
}

impl Settings {
    pub(crate) fn try_reload_from(&mut self, new_raw: SettingsRaw) -> Result<()> {
        let mut current_raw = (*self.settings_raw).clone();

        // reload extensions config
        current_raw.extensions = new_raw.extensions;

        // reload turn settings
        current_raw.turn = new_raw.turn;

        // reload metrics
        current_raw.metrics = new_raw.metrics;

        // reload avatar
        current_raw.avatar = new_raw.avatar;

        // reload call in
        current_raw.call_in = new_raw.call_in;

        self.settings_raw = Arc::new(current_raw);

        Ok(())
    }
}

impl TryFrom<Arc<SettingsRaw>> for Settings {
    type Error = SettingsError;

    fn try_from(settings_raw: Arc<SettingsRaw>) -> Result<Self, Self::Error> {
        Ok(Settings { settings_raw })
    }
}
