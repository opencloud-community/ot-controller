// SPDX-FileCopyrightText: OpenTalk GmbH <mail@opentalk.eu>
//
// SPDX-License-Identifier: EUPL-1.2

use snafu::ResultExt as _;
use warning_source::WarningSource;

use super::SettingsProvider;
use crate::{settings_error::DeserializeConfigSnafu, Result, SettingsRaw};

mod warning_source;

impl SettingsProvider {
    pub(super) fn load_raw(file_name: &str) -> Result<SettingsRaw> {
        use config::{Config, Environment, File, FileFormat};

        let config = Config::builder()
            .add_source(File::new(file_name, FileFormat::Toml))
            .add_source(WarningSource::new(
                Environment::with_prefix("K3K_CTRL")
                    .prefix_separator("_")
                    .separator("__"),
            ))
            .add_source(
                Environment::with_prefix("OPENTALK_CTRL")
                    .prefix_separator("_")
                    .separator("__"),
            )
            .build()?;

        let settings_raw: SettingsRaw =
            serde_path_to_error::deserialize(config).context(DeserializeConfigSnafu {
                file_name: file_name.to_owned(),
            })?;
        Self::warn_about_deprecated_items(&settings_raw);

        Ok(settings_raw)
    }

    fn warn_about_deprecated_items(raw: &SettingsRaw) {
        use owo_colors::OwoColorize as _;

        if raw.extensions.contains_key("room_server") {
            anstream::eprintln!(
                "{}: Found an obsolete {room_server} (janus) configuration section.\n\
                 {}: This section is no longer needed, please remove it and add a {livekit} section instead.",
                "DEPRECATION WARNING".yellow().bold(),
                "NOTE".green(),
                room_server = "room_server".bold(),
                livekit = "livekit".bold(),
            );
        }

        if raw.keycloak.is_some() {
            anstream::eprintln!(
                "{}: Found an obsolete {keycloak} (oidc) configuration section.\n\
                 {}: This section is deprecated, please replace it with the newly introduced {oidc} and {user_search} sections.",
                "DEPRECATION WARNING".yellow().bold(),
                "NOTE".green(),
                keycloak = "keycloak".bold(),
                oidc = "oidc".bold(),
                user_search = "user_search".bold(),
            );
        }

        if raw.turn.is_some() {
            anstream::eprintln!(
                "{}: Found an obsolete {turn} server configuration.\n\
                 {}: The {turn} config section as well as the related {endpoint} endpoint will be removed in the future.",
                "DEPRECATION WARNING".yellow().bold(),
                "NOTE".green(),
                turn = "turn".bold(),
                endpoint = "/turn".bold()
            );
        }

        if raw.reports.is_some() {
            anstream::eprintln!(
                "{}: Found an obsolete {reports} configuration section.\n\
                 {}: This section is deprecated and will be reintroduced in a different form in the future.",
                "DEPRECATION WARNING".yellow().bold(),
                "NOTE".green(),
                reports = "reports".bold(),
            );
        }
    }
}

#[cfg(test)]
mod tests {
    use std::env;

    use pretty_assertions::assert_eq;

    use super::*;

    #[test]
    fn settings_env_vars_overwrite_config() -> Result<()> {
        env::remove_var("OPENTALK_CTRL_DATABASE__URL");
        env::remove_var("OPENTALK_CTRL_HTTP__PORT");
        env::remove_var("OPENTALK_CTRL_DEFAULTS__SCREEN_SHARE_REQUIRES_PERMISSION");

        // Sanity check
        let settings = SettingsProvider::load_raw("../../extra/example.toml")?;

        assert_eq!(
            settings.database.url,
            "postgres://postgres:password123@localhost:5432/opentalk"
        );
        assert!(settings.http.is_none());

        // Set environment variables to overwrite default config file
        let env_db_url = "postgres://envtest:password@localhost:5432/opentalk".to_string();
        let env_http_port: u16 = 8000;
        let screen_share_requires_permission = true;
        env::set_var("OPENTALK_CTRL_DATABASE__URL", &env_db_url);
        env::set_var("OPENTALK_CTRL_HTTP__PORT", env_http_port.to_string());
        env::set_var(
            "OPENTALK_CTRL_DEFAULTS__SCREEN_SHARE_REQUIRES_PERMISSION",
            screen_share_requires_permission.to_string(),
        );

        let settings = SettingsProvider::load_raw("../../extra/example.toml")?;

        assert_eq!(settings.database.url, env_db_url);
        assert_eq!(settings.http.as_ref().unwrap().port, Some(env_http_port));
        assert_eq!(
            settings.defaults.screen_share_requires_permission,
            screen_share_requires_permission
        );

        Ok(())
    }
}
