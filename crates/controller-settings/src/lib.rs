// SPDX-FileCopyrightText: OpenTalk GmbH <mail@opentalk.eu>
//
// SPDX-License-Identifier: EUPL-1.2

//! Contains the application settings.
//!
//! The application settings are set with a TOML config file. Settings specified in the config file
//! can be overwritten by environment variables. To do so, set an environment variable
//! with the prefix `OPENTALK_CTRL_` followed by the field names you want to set. Nested fields are separated by two underscores `__`.
//! ```sh
//! OPENTALK_CTRL_<field>__<field-of-field>...
//! ```
//!
//! # Example
//!
//! set the `database.url` field:
//! ```sh
//! OPENTALK_CTRL_DATABASE__URL=postgres://postgres:password123@localhost:5432/opentalk
//! ```
//!
//! So the field 'database.max_connections' would resolve to:
//! ```sh
//! OPENTALK_CTRL_DATABASE__MAX_CONNECTIONS=5
//! ```
//!
//! # Note
//!
//! Fields set via environment variables do not affect the underlying config file.
//!
//! # Implementation Details:
//!
//! Setting categories, in which all properties implement a default value, should also implement the [`Default`] trait.

use anyhow::{anyhow, Context, Result};
use arc_swap::ArcSwap;
use config::{Config, Environment, File, FileFormat};
use openidconnect::{ClientId, ClientSecret};
use rustc_hash::FxHashSet;
use serde::{Deserialize, Deserializer};
use std::collections::{HashMap, HashSet};
use std::convert::TryFrom;
use std::path::PathBuf;
use std::sync::Arc;
use std::time::Duration;
use url::Url;

pub type SharedSettings = Arc<ArcSwap<Settings>>;

#[derive(Debug, Clone, Deserialize)]
pub struct Settings {
    pub database: Database,
    pub keycloak: Keycloak,
    pub http: Http,
    #[serde(default)]
    pub turn: Option<Turn>,
    #[serde(default)]
    pub stun: Option<Stun>,
    #[serde(default)]
    pub redis: RedisConfig,
    #[serde(default)]
    pub rabbit_mq: RabbitMqConfig,
    #[serde(default)]
    pub logging: Logging,
    #[serde(default)]
    pub authz: Authz,
    #[serde(default)]
    pub avatar: Avatar,
    #[serde(default)]
    pub metrics: Metrics,
    #[serde(default)]
    pub etherpad: Option<Etherpad>,

    #[serde(default)]
    pub spacedeck: Option<Spacedeck>,

    #[serde(default)]
    pub shared_folder: Option<SharedFolder>,

    #[serde(default)]
    pub call_in: Option<CallIn>,

    #[serde(default)]
    pub defaults: Defaults,

    #[serde(default)]
    pub endpoints: Endpoints,

    pub minio: MinIO,

    #[serde(default)]
    pub tenants: Tenants,

    #[serde(default)]
    pub tariffs: Tariffs,

    #[serde(flatten)]
    pub extensions: HashMap<String, config::Value>,
}

#[derive(Debug, Clone)]
struct WarningSource<T: Clone>(T);

impl<T> config::Source for WarningSource<T>
where
    T: config::Source + Send + Sync + Clone + 'static,
{
    fn clone_into_box(&self) -> Box<dyn config::Source + Send + Sync> {
        Box::new((*self).clone())
    }

    fn collect(&self) -> Result<config::Map<String, config::Value>, config::ConfigError> {
        let values = self.0.collect()?;
        if !values.is_empty() {
            use owo_colors::OwoColorize as _;

            anstream::eprintln!(
                "{}: The following environment variables have been deprecated and \
                will not work in a future release. Please change them as suggested below:",
                "DEPRECATION WARNING".yellow().bold(),
            );

            for key in values.keys() {
                let env_var = key.replace('.', "__").to_uppercase();
                anstream::eprintln!(
                    "{}: rename environment variable {} to {}",
                    "DEPRECATION WARNING".yellow().bold(),
                    format!("K3K_CTRL_{}", env_var).yellow(),
                    format!("OPENTALK_CTRL_{}", env_var).green().bold(),
                );
            }
        }

        Ok(values)
    }
}

impl Settings {
    /// Creates a new Settings instance from the provided TOML file.
    /// Specific fields can be set or overwritten with environment variables (See struct level docs for more details).
    pub fn load(file_name: &str) -> Result<Self> {
        let config = Config::builder()
            .add_source(File::new(file_name, FileFormat::Toml))
            .add_source(WarningSource(
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

        serde_path_to_error::deserialize(config)
            .map_err(|e| anyhow!("{} for `{}`", e.inner(), e.path()))
            .with_context(|| {
                format!("Failed to apply configuration from {file_name} or environment")
            })
    }
}

#[derive(Debug, Clone, Deserialize)]
pub struct Database {
    pub url: String,
    #[serde(default = "default_max_connections")]
    pub max_connections: u32,
}

fn default_max_connections() -> u32 {
    100
}

/// Settings for Keycloak
#[derive(Debug, Clone, Deserialize)]
pub struct Keycloak {
    pub base_url: Url,
    pub realm: String,
    pub client_id: ClientId,
    pub client_secret: ClientSecret,
    pub external_id_user_attribute_name: Option<String>,
}

#[derive(Debug, Clone, Deserialize)]
pub struct Http {
    #[serde(default = "default_http_port")]
    pub port: u16,
    #[serde(default)]
    pub tls: Option<HttpTls>,
}

impl Default for Http {
    fn default() -> Self {
        Self {
            port: default_http_port(),
            tls: None,
        }
    }
}

const fn default_http_port() -> u16 {
    11311
}

#[derive(Debug, Clone, Deserialize)]
pub struct HttpTls {
    pub certificate: PathBuf,
    pub private_key: PathBuf,
}

#[derive(Default, Debug, Clone, Deserialize)]
pub struct Logging {
    pub default_directives: Option<Vec<String>>,

    pub otlp_tracing_endpoint: Option<String>,

    pub service_name: Option<String>,

    pub service_namespace: Option<String>,

    pub service_instance_id: Option<String>,
}

#[derive(Debug, Clone, Deserialize)]
pub struct Turn {
    /// How long should a credential pair be valid, in seconds
    #[serde(
        deserialize_with = "duration_from_secs",
        default = "default_turn_credential_lifetime"
    )]
    pub lifetime: Duration,
    /// List of configured TURN servers.
    pub servers: Vec<TurnServer>,
}

impl Default for Turn {
    fn default() -> Self {
        Self {
            lifetime: default_turn_credential_lifetime(),
            servers: vec![],
        }
    }
}

fn default_turn_credential_lifetime() -> Duration {
    Duration::from_secs(60)
}

#[derive(Debug, Clone, Deserialize)]
pub struct TurnServer {
    // TURN URIs for this TURN server following rfc7065
    pub uris: Vec<String>,
    pub pre_shared_key: String,
}

#[derive(Clone, Debug, Deserialize)]
pub struct Stun {
    // STUN URIs for this TURN server following rfc7065
    pub uris: Vec<String>,
}

#[derive(Debug, Clone, Deserialize)]
pub struct RedisConfig {
    #[serde(default = "redis_default_url")]
    pub url: url::Url,
}

impl Default for RedisConfig {
    fn default() -> Self {
        Self {
            url: redis_default_url(),
        }
    }
}

fn redis_default_url() -> url::Url {
    url::Url::try_from("redis://localhost:6379/").expect("Invalid default redis URL")
}

#[derive(Debug, Clone, Deserialize)]
pub struct RabbitMqConfig {
    #[serde(default = "rabbitmq_default_url")]
    pub url: String,
    #[serde(default = "rabbitmq_default_min_connections")]
    pub min_connections: u32,
    #[serde(default = "rabbitmq_default_max_channels")]
    pub max_channels_per_connection: u32,
    /// Mail sending is disabled when this is None
    #[serde(default)]
    pub mail_task_queue: Option<String>,

    /// Recording is disabled if this isn't set
    #[serde(default)]
    pub recording_task_queue: Option<String>,
}

impl Default for RabbitMqConfig {
    fn default() -> Self {
        Self {
            url: rabbitmq_default_url(),
            min_connections: rabbitmq_default_min_connections(),
            max_channels_per_connection: rabbitmq_default_max_channels(),
            mail_task_queue: None,
            recording_task_queue: None,
        }
    }
}

fn rabbitmq_default_url() -> String {
    "amqp://guest:guest@localhost:5672".to_owned()
}

fn rabbitmq_default_min_connections() -> u32 {
    10
}

fn rabbitmq_default_max_channels() -> u32 {
    100
}

#[derive(Clone, Debug, Deserialize)]
pub struct Authz {
    /// Authz reload interval in seconds
    #[serde(
        deserialize_with = "duration_from_secs",
        default = "default_authz_reload_interval"
    )]
    pub reload_interval: Duration,
}

impl Default for Authz {
    fn default() -> Self {
        Self {
            reload_interval: default_authz_reload_interval(),
        }
    }
}

fn default_authz_reload_interval() -> Duration {
    Duration::from_secs(10)
}

#[derive(Clone, Debug, Deserialize)]
pub struct Etherpad {
    pub url: url::Url,
    pub api_key: String,
}

#[derive(Clone, Debug, Deserialize)]
pub struct Spacedeck {
    pub url: url::Url,
    pub api_key: String,
}

#[derive(Clone, Debug, Deserialize, PartialEq, Eq)]
#[serde(tag = "provider", rename_all = "snake_case")]
pub enum SharedFolder {
    Nextcloud {
        url: url::Url,
        username: String,
        password: String,
        #[serde(default)]
        directory: String,
        #[serde(default)]
        expiry: Option<u64>,
    },
}

fn duration_from_secs<'de, D>(deserializer: D) -> Result<Duration, D::Error>
where
    D: Deserializer<'de>,
{
    let duration: u64 = Deserialize::deserialize(deserializer)?;

    Ok(Duration::from_secs(duration))
}

#[derive(Clone, Debug, Deserialize)]
pub struct Avatar {
    #[serde(default = "default_libravatar_url")]
    pub libravatar_url: String,
}

impl Default for Avatar {
    fn default() -> Self {
        Self {
            libravatar_url: default_libravatar_url(),
        }
    }
}

fn default_libravatar_url() -> String {
    "https://seccdn.libravatar.org/avatar/".into()
}

#[derive(Clone, Debug, Deserialize)]
pub struct CallIn {
    pub tel: String,
    pub enable_phone_mapping: bool,
    pub default_country_code: phonenumber::country::Id,
}

/// The namespace that is used by default
pub const DEFAULT_NAMESPACE: &str = "core";

/// The namespace separator
pub const NAMESPACE_SEPARATOR: &str = "::";

#[derive(Clone, Default, Debug, Deserialize)]
pub struct Defaults {
    #[serde(default = "default_user_language")]
    pub user_language: String,
    #[serde(default)]
    pub screen_share_requires_permission: bool,
    #[serde(default)]
    disabled_features: HashSet<String>,
}

impl Defaults {
    pub fn disabled_features(&self) -> HashSet<String> {
        self.disabled_features
            .iter()
            .map(|feature| {
                if feature.contains(NAMESPACE_SEPARATOR) {
                    feature.to_owned()
                } else {
                    format!("{DEFAULT_NAMESPACE}{NAMESPACE_SEPARATOR}{feature}")
                }
            })
            .collect()
    }
}

fn default_user_language() -> String {
    "en-US".into()
}

#[derive(Clone, Default, Debug, Deserialize)]
pub struct Endpoints {
    #[serde(default)]
    pub disable_users_find: bool,
    #[serde(default)]
    pub users_find_use_kc: bool,
    #[serde(default)]
    pub event_invite_external_email_address: bool,
    #[serde(default)]
    pub disallow_custom_display_name: bool,
}

#[derive(Clone, Debug, Deserialize)]
pub struct MinIO {
    pub uri: String,
    pub bucket: String,
    pub access_key: String,
    pub secret_key: String,
}

#[derive(Debug, Default, Clone, Deserialize)]
pub struct Metrics {
    pub allowlist: Vec<cidr::IpInet>,
}

#[derive(Debug, Clone, Deserialize)]
#[serde(rename_all = "snake_case", tag = "assignment")]
pub enum TenantAssignment {
    Static {
        static_tenant_id: String,
    },
    ByExternalTenantId {
        #[serde(default = "default_external_tenant_id_user_attribute_name")]
        external_tenant_id_user_attribute_name: String,
    },
}

fn default_external_tenant_id_user_attribute_name() -> String {
    "tenant_id".to_owned()
}

impl Default for TenantAssignment {
    fn default() -> Self {
        Self::Static {
            static_tenant_id: String::from("OpenTalkDefaultTenant"),
        }
    }
}

#[derive(Default, Debug, Clone, Deserialize)]
pub struct Tenants {
    #[serde(default, flatten)]
    pub assignment: TenantAssignment,
}

#[derive(Debug, Clone, Deserialize)]
#[serde(rename_all = "snake_case", tag = "assignment")]
pub enum TariffAssignment {
    Static { static_tariff_name: String },
    ByExternalTariffId,
}

impl Default for TariffAssignment {
    fn default() -> Self {
        Self::Static {
            static_tariff_name: String::from("OpenTalkDefaultTariff"),
        }
    }
}

#[derive(Default, Debug, Clone, Deserialize)]
pub struct TariffStatusMapping {
    pub downgraded_tariff_name: String,
    pub default: FxHashSet<String>,
    pub paid: FxHashSet<String>,
    pub downgraded: FxHashSet<String>,
}

#[derive(Default, Debug, Clone, Deserialize)]
pub struct Tariffs {
    #[serde(default, flatten)]
    pub assignment: TariffAssignment,

    #[serde(default)]
    pub status_mapping: Option<TariffStatusMapping>,
}

#[cfg(test)]
mod test {
    use super::*;
    use pretty_assertions::assert_eq;
    use serde_json::json;
    use std::env;

    #[test]
    fn settings_env_vars_overwrite_config() -> Result<()> {
        // Sanity check
        let settings = Settings::load("../../extra/example.toml")?;

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

        let settings = Settings::load("../../extra/example.toml")?;

        assert_eq!(settings.database.url, env_db_url);
        assert_eq!(settings.http.port, env_http_port);
        assert_eq!(
            settings.defaults.screen_share_requires_permission,
            screen_share_requires_permission
        );

        Ok(())
    }

    #[test]
    fn shared_folder_provider_nextcloud() {
        let shared_folder = SharedFolder::Nextcloud {
            url: "https://nextcloud.example.org/".parse().unwrap(),
            username: "exampleuser".to_string(),
            password: "v3rys3cr3t".to_string(),
            directory: "meetings/opentalk".to_string(),
            expiry: Some(34),
        };
        let json = json!({
            "provider": "nextcloud",
            "url": "https://nextcloud.example.org/",
            "username": "exampleuser",
            "password": "v3rys3cr3t",
            "directory": "meetings/opentalk",
            "expiry": 34,
        });

        assert_eq!(
            serde_json::from_value::<SharedFolder>(json).unwrap(),
            shared_folder
        );
    }
}
