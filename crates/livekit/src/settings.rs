// SPDX-FileCopyrightText: OpenTalk GmbH <mail@opentalk.eu>
//
// SPDX-License-Identifier: EUPL-1.2

use config::ConfigError;
use serde::Deserialize;

#[derive(Deserialize)]
pub struct LivekitSettings {
    pub url: String,
    pub api_key: String,
    pub api_secret: String,
}

impl LivekitSettings {
    pub fn extract(
        settings: &opentalk_controller_settings::Settings,
    ) -> Result<Option<Self>, ConfigError> {
        let Some(value) = settings.extensions.get("livekit").cloned() else {
            return Ok(None);
        };

        Some(value.try_deserialize()).transpose()
    }
}
