// SPDX-FileCopyrightText: OpenTalk GmbH <mail@opentalk.eu>
//
// SPDX-License-Identifier: EUPL-1.2

use std::sync::Arc;

use arc_swap::ArcSwap;

use crate::{Result, Settings};

/// A struct for loading and holding the runtime settings.
#[derive(Debug, Clone)]
pub struct SettingsProvider {
    settings: Arc<ArcSwap<Settings>>,
}

impl SettingsProvider {
    /// Load the settings from a TOML file.
    ///
    /// This will succeed in case the file could be loaded successfully.
    /// Environment variables in the `OPENTALK_CTRL_*` pattern are considiered
    /// and will override the settings found in the file.
    pub fn load(file_name: &str) -> Result<Self> {
        let settings = Settings::load(file_name)?;
        Ok(Self::new(Arc::new(settings)))
    }

    /// Create a new [`SettingsProvider`] with settings that are already loaded.
    pub fn new(settings: Arc<Settings>) -> Self {
        Self {
            settings: Arc::new(ArcSwap::new(settings)),
        }
    }

    /// Get an `[Arc]` holding the current runtime settings.
    ///
    /// The returned settings will remain unchanged even if the settings are
    /// reloaded by the [`SettingsProvider`]. A new [`Arc`] will be created
    /// internally by the `reload` function. This allows consistent use of a
    /// "snapshot" inside a function by calling `get` once, and then using
    /// the returned value.
    pub fn get(&self) -> Arc<Settings> {
        self.settings.load_full()
    }

    /// Reload the settings from a TOML file.
    ///
    /// Not all settings are used, as most of the settings are not reloadable
    /// while the controller is running.
    ///
    /// This will succeed in case the file could be loaded successfully.
    /// Environment variables in the `OPENTALK_CTRL_*` pattern are considiered
    /// and will override the settings found in the file.
    ///
    /// If loading the settings fails, an error is returned from this function
    /// and stored configuration will remain unchanged.
    ///
    /// Any "snapshots" handed out to callers by the `get` function remain
    /// unchanged, so wherever these are used, the values will not change.
    /// Because an `Arc` was given to these callers, the value will be freed
    /// once the last reference to it has been dropped.
    pub fn reload(&self, config_path: &str) -> Result<()> {
        let new_settings = Settings::load(config_path)?;
        let mut current_settings = (*self.settings.load_full()).clone();

        // reload extensions config
        current_settings.extensions = new_settings.extensions;

        // reload turn settings
        current_settings.turn = new_settings.turn;

        // reload metrics
        current_settings.metrics = new_settings.metrics;

        // reload avatar
        current_settings.avatar = new_settings.avatar;

        // reload call in
        current_settings.call_in = new_settings.call_in;

        // replace the shared settings with the modified ones
        self.settings.store(Arc::new(current_settings));

        Ok(())
    }
}

#[cfg(test)]
mod tests {
    use std::{fs::File, io::Write as _};

    use openidconnect::{ClientId, ClientSecret};
    use pretty_assertions::{assert_eq, assert_matches, assert_ne};
    use tempfile::tempdir;

    use super::SettingsProvider;
    use crate::{
        settings_file::{
            database::default_max_connections, Authz, Avatar, ControllerOidcConfiguration,
            Database, Defaults, Endpoints, Extensions, FrontendOidcConfiguration, Http, Logging,
            Metrics, MinIO, Oidc, OidcConfiguration, OidcController, OidcFrontend, RabbitMqConfig,
            UserSearch, UserSearchBackend, UserSearchConfiguration, UsersFindBehavior,
        },
        LiveKitSettings, OidcAndUserSearchConfiguration, Settings, SettingsError, Tariffs, Tenants,
    };

    const MINIMUM_CONFIG_TOML: &str = r#"
        [database]
        url = "postgres://postgres:password123@localhost:5432/opentalk"

        [http]

        [minio]
        uri = "http://localhost:9555"
        bucket = "controller"
        access_key = "minioadmin"
        secret_key = "minioadmin"

        [livekit]
        public_url = "ws://localhost:7880"
        service_url = "http://localhost:7880"
        api_key = "devkey"
        api_secret = "secret"

        [oidc]
        authority = "http://localhost:8080/realms/opentalk"

        [oidc.frontend]
        client_id = "Webapp"

        [oidc.controller]
        client_id = "Controller"
        client_secret = "mysecret"

        [user_search]
        backend = "keycloak_webapi"
        api_base_url = "http://localhost:8080/admin/realms/opentalk"
        users_find_behavior = "disabled"
        "#;

    fn minimum_config() -> Settings {
        Settings {
            database: Database {
                url: "postgres://postgres:password123@localhost:5432/opentalk".to_string(),
                max_connections: default_max_connections(),
            },
            keycloak: None,
            oidc: Some(Oidc {
                authority: "http://localhost:8080/realms/opentalk"
                    .parse()
                    .expect("must be a valid url"),
                frontend: OidcFrontend {
                    authority: None,
                    client_id: ClientId::new("Webapp".to_string()),
                },
                controller: OidcController {
                    authority: None,
                    client_id: ClientId::new("Controller".to_string()),
                    client_secret: ClientSecret::new("mysecret".to_string()),
                },
            }),
            user_search: Some(UserSearch {
                backend: UserSearchBackend::KeycloakWebapi,
                api_base_url: "http://localhost:8080/admin/realms/opentalk"
                    .parse()
                    .expect("must be a valid url"),
                client_id: None,
                client_secret: None,
                external_id_user_attribute_name: None,
                users_find_behavior: UsersFindBehavior::Disabled,
            }),
            oidc_and_user_search: OidcAndUserSearchConfiguration {
                oidc: OidcConfiguration {
                    frontend: FrontendOidcConfiguration {
                        auth_base_url: "http://localhost:8080/realms/opentalk"
                            .parse()
                            .expect("must be a valid url"),
                        client_id: ClientId::new("Webapp".to_string()),
                    },
                    controller: ControllerOidcConfiguration {
                        auth_base_url: "http://localhost:8080/realms/opentalk"
                            .parse()
                            .expect("must be a valid url"),
                        client_id: ClientId::new("Controller".to_string()),
                        client_secret: ClientSecret::new("mysecret".to_string()),
                    },
                },
                user_search: UserSearchConfiguration {
                    backend: UserSearchBackend::KeycloakWebapi,
                    api_base_url: "http://localhost:8080/admin/realms/opentalk"
                        .parse()
                        .expect("must be a valid url"),
                    client_id: ClientId::new("Controller".to_string()),
                    client_secret: ClientSecret::new("mysecret".to_string()),
                    external_id_user_attribute_name: None,
                    users_find_behavior: UsersFindBehavior::Disabled,
                },
            },
            http: Http::default(),
            turn: None,
            stun: None,
            redis: None,
            rabbit_mq: RabbitMqConfig::default(),
            logging: Logging::default(),
            authz: Authz::default(),
            avatar: Avatar::default(),
            metrics: Metrics::default(),
            etcd: None,
            etherpad: None,
            spacedeck: None,
            subroom_audio: None,
            reports: None,
            shared_folder: None,
            call_in: None,
            defaults: Defaults::default(),
            endpoints: Endpoints::default(),
            minio: MinIO {
                uri: "http://localhost:9555"
                    .parse()
                    .expect("must be a valid url"),
                bucket: "controller".to_string(),
                access_key: "minioadmin".to_string(),
                secret_key: "minioadmin".to_string(),
            },
            monitoring: None,
            tenants: Tenants::default(),
            tariffs: Tariffs::default(),
            livekit: LiveKitSettings {
                public_url: "ws://localhost:7880".to_string(),
                service_url: "http://localhost:7880".to_string(),
                api_key: "devkey".to_string(),
                api_secret: "secret".to_string(),
            },
            extensions: Extensions::default(),
        }
    }

    #[test]
    fn load_minimal() {
        let tempdir = tempdir().unwrap();

        let path = tempdir.path().join("controller.toml");

        {
            let mut file = File::create(&path).unwrap();
            writeln!(file, "{MINIMUM_CONFIG_TOML}").expect("temp file should be writable");
        }

        let settings_provider =
            SettingsProvider::load(path.to_str().expect("valid file path expected"))
                .expect("valid configuration expected");

        assert_eq!(&(*settings_provider.get()), &minimum_config());
    }

    #[test]
    fn load_invalid() {
        let tempdir = tempdir().unwrap();

        let path = tempdir.path().join("controller.toml");

        {
            // Create an empty file which will result in an invalid definition
            let _file = File::create(&path).unwrap();
        }

        assert_matches!(
            SettingsProvider::load(path.to_str().expect("valid file path expected")),
            Err(SettingsError::DeserializeConfig {
                file_name: _,
                source: _
            })
        );
    }

    #[test]
    fn reload() {
        let tempdir = tempdir().unwrap();

        let modified_path = tempdir.path().join("controller_modified.toml");
        let minimal_path = tempdir.path().join("controller_minimal.toml");

        {
            let mut file = File::create(&modified_path).unwrap();
            writeln!(
                file,
                r#"
                {MINIMUM_CONFIG_TOML}

                [call_in]
                tel = "+55667788"
                enable_phone_mapping = true
                default_country_code = "DE"
                "#
            )
            .expect("temp file should be writable");
        }
        {
            let mut file = File::create(&minimal_path).unwrap();
            writeln!(file, "{MINIMUM_CONFIG_TOML}").expect("temp file should be writable");
        }

        let settings_provider =
            SettingsProvider::load(modified_path.to_str().expect("valid file path expected"))
                .expect("valid configuration expected");

        assert_ne!(&(*settings_provider.get()), &minimum_config());

        settings_provider
            .reload(minimal_path.to_str().expect("valid file path expected"))
            .expect("reload is expected to succeed");

        assert_eq!(&(*settings_provider.get()), &minimum_config());
    }

    #[test]
    fn reload_invalid() {
        let tempdir = tempdir().unwrap();

        let invalid_path = tempdir.path().join("controller_invalid.toml");
        let minimal_path = tempdir.path().join("controller_minimal.toml");

        {
            // Create an empty file which will result in an invalid definition
            let _file = File::create(&invalid_path).unwrap();
        }
        {
            let mut file = File::create(&minimal_path).unwrap();
            writeln!(file, "{MINIMUM_CONFIG_TOML}").expect("temp file should be writable");
        }

        let settings_provider =
            SettingsProvider::load(minimal_path.to_str().expect("valid file path expected"))
                .expect("valid configuration expected");

        assert_eq!(&(*settings_provider.get()), &minimum_config());

        assert_matches!(
            settings_provider.reload(invalid_path.to_str().expect("valid file path expected")),
            Err(SettingsError::DeserializeConfig {
                file_name: _,
                source: _
            })
        );

        assert_eq!(&(*settings_provider.get()), &minimum_config());
    }
}
