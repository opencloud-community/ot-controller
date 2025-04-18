// SPDX-FileCopyrightText: OpenTalk GmbH <mail@opentalk.eu>
//
// SPDX-License-Identifier: EUPL-1.2

use snafu::ResultExt as _;
use warning_source::WarningSource;

use super::SettingsProvider;
use crate::{
    settings_error::DeserializeConfigSnafu, settings_file::SettingsLoading, Result, SettingsRaw,
};

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

        let initial: SettingsLoading<()> =
            serde_path_to_error::deserialize(config).context(DeserializeConfigSnafu {
                file_name: file_name.to_owned(),
            })?;
        Self::warn_about_deprecated_items(&initial);

        let oidc_and_user_search = initial.build_oidc_and_user_search_configuration()?;

        Ok(SettingsRaw {
            oidc_and_user_search,
            database: initial.database,
            keycloak: initial.keycloak,
            oidc: initial.oidc,
            user_search: initial.user_search,
            http: initial.http,
            turn: initial.turn,
            stun: initial.stun,
            redis: initial.redis,
            rabbit_mq: initial.rabbit_mq,
            logging: initial.logging,
            authz: initial.authz,
            avatar: initial.avatar,
            metrics: initial.metrics,
            etcd: initial.etcd,
            etherpad: initial.etherpad,
            spacedeck: initial.spacedeck,
            reports: initial.reports,
            subroom_audio: initial.subroom_audio,
            shared_folder: initial.shared_folder,
            call_in: initial.call_in,
            defaults: initial.defaults,
            endpoints: initial.endpoints,
            minio: initial.minio,
            monitoring: initial.monitoring,
            tenants: initial.tenants,
            tariffs: initial.tariffs,
            livekit: initial.livekit,
            extensions: initial.extensions,
        })
    }

    fn warn_about_deprecated_items(raw: &SettingsLoading<()>) {
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
        // Sanity check
        let settings = SettingsProvider::load_raw("../../extra/example.toml")?;

        assert_eq!(
            settings.database.url,
            "postgres://postgres:password123@localhost:5432/opentalk"
        );
        assert_eq!(settings.http.port, 11311u16);

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
        assert_eq!(settings.http.port, env_http_port);
        assert_eq!(
            settings.defaults.screen_share_requires_permission,
            screen_share_requires_permission
        );

        Ok(())
    }
}
