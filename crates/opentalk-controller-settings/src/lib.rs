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
//!
//! [`SettingsLoading<()>`] contains incomplete fields and is an intermediate after
//! loading the raw settings inside [`SettingsProvider::load`]. The final struct with all loaded fields
//! is [`Settings`] (an alias for [`SettingsLoading<OidcAndUserSearchConfiguration>`]) which contains all loaded fields.

use std::{collections::BTreeSet, convert::TryFrom, time::Duration};

use opentalk_types_common::{features::ModuleFeatureId, users::Language};
use rustc_hash::FxHashSet;
use serde::{Deserialize, Deserializer};
use settings_file::{OidcAndUserSearchConfiguration, SettingsLoading};

pub mod settings_file;

mod settings_error;
mod settings_provider;

pub use settings_error::SettingsError;
pub use settings_provider::SettingsProvider;

type Result<T, E = SettingsError> = std::result::Result<T, E>;

pub type Settings = SettingsLoading<OidcAndUserSearchConfiguration>;

#[derive(Debug, Clone, PartialEq, Eq, Deserialize)]
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

#[derive(Debug, Clone, PartialEq, Eq, Deserialize)]
pub struct TurnServer {
    // TURN URIs for this TURN server following rfc7065
    pub uris: Vec<String>,
    pub pre_shared_key: String,
}

#[derive(Clone, Debug, PartialEq, Eq, Deserialize)]
pub struct Stun {
    // STUN URIs for this TURN server following rfc7065
    pub uris: Vec<String>,
}

#[derive(Debug, Clone, PartialEq, Eq, Deserialize)]
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

#[derive(Debug, Clone, PartialEq, Eq, Deserialize)]
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

#[derive(Clone, Debug, PartialEq, Eq, Deserialize)]
pub struct Authz {
    #[serde(default = "authz_default_synchronize_controller")]
    pub synchronize_controllers: bool,
}

impl Default for Authz {
    fn default() -> Self {
        Self {
            synchronize_controllers: authz_default_synchronize_controller(),
        }
    }
}

fn authz_default_synchronize_controller() -> bool {
    true
}

#[derive(Clone, Debug, PartialEq, Eq, Deserialize)]
pub struct Etcd {
    pub urls: Vec<url::Url>,
}

#[derive(Clone, Debug, PartialEq, Eq, Deserialize)]
pub struct Etherpad {
    pub url: url::Url,
    pub api_key: String,
}

#[derive(Clone, Debug, PartialEq, Eq, Deserialize)]
pub struct Spacedeck {
    pub url: url::Url,
    pub api_key: String,
}

#[derive(Clone, Debug, PartialEq, Eq, Deserialize)]
pub struct SubroomAudio {
    #[serde(default)]
    pub enable_whisper: bool,
}

#[derive(Default, Clone, Debug, Deserialize, PartialEq, Eq)]
pub struct Reports {
    #[serde(default)]
    pub template: ReportsTemplate,
}

#[derive(Clone, Debug, Deserialize, Default, PartialEq, Eq)]
#[serde(rename_all = "snake_case")]
pub enum ReportsTemplate {
    /// Use the Template included with the application.
    #[default]
    BuiltIn,

    /// Use the Template provided by the user configuration.
    Inline(String),
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

#[derive(Clone, Debug, PartialEq, Eq, Deserialize)]
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

#[derive(Clone, Debug, PartialEq, Eq, Deserialize)]
pub struct CallIn {
    pub tel: String,
    pub enable_phone_mapping: bool,
    pub default_country_code: phonenumber::country::Id,
}

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

#[derive(Clone, Default, Debug, PartialEq, Eq, Deserialize)]
pub struct Endpoints {
    pub disable_users_find: Option<bool>,
    pub users_find_use_kc: Option<bool>,
    #[serde(default)]
    pub event_invite_external_email_address: bool,
    #[serde(default)]
    pub disallow_custom_display_name: bool,
    #[serde(default)]
    pub disable_openapi: bool,
}

#[derive(Clone, Debug, PartialEq, Eq, Deserialize)]
pub struct MinIO {
    pub uri: String,
    pub bucket: String,
    pub access_key: String,
    pub secret_key: String,
}

#[derive(Debug, Default, Clone, PartialEq, Eq, Deserialize)]
pub struct Metrics {
    pub allowlist: Vec<cidr::IpInet>,
}

#[derive(Debug, Clone, PartialEq, Eq, Deserialize)]
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

#[derive(Default, Debug, Clone, PartialEq, Eq, Deserialize)]
pub struct Tenants {
    #[serde(default, flatten)]
    pub assignment: TenantAssignment,
}

#[derive(Debug, Clone, PartialEq, Eq, Deserialize)]
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

#[derive(Default, Debug, Clone, PartialEq, Eq, Deserialize)]
pub struct TariffStatusMapping {
    pub downgraded_tariff_name: String,
    pub default: FxHashSet<String>,
    pub paid: FxHashSet<String>,
    pub downgraded: FxHashSet<String>,
}

#[derive(Default, Debug, Clone, PartialEq, Eq, Deserialize)]
pub struct Tariffs {
    #[serde(default, flatten)]
    pub assignment: TariffAssignment,

    #[serde(default)]
    pub status_mapping: Option<TariffStatusMapping>,
}

#[derive(Debug, Clone, PartialEq, Eq, Deserialize)]
pub struct LiveKitSettings {
    pub api_key: String,
    pub api_secret: String,
    pub public_url: String,

    // for backwards compatibility
    #[serde(alias = "url")]
    pub service_url: String,
}

#[cfg(test)]
mod tests {
    use pretty_assertions::assert_eq;
    use serde_json::json;

    use super::*;

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

    #[test]
    fn meeting_report_settings() {
        let toml_settings: Reports = toml::from_str(
            r#"
        url = "http://localhost"
        "#,
        )
        .unwrap();
        assert_eq!(
            toml_settings,
            Reports {
                template: ReportsTemplate::BuiltIn
            }
        );

        let toml_settings: Reports = toml::from_str(
            r#"
        url = "http://localhost"
        template.inline = "lorem ipsum"
        "#,
        )
        .unwrap();
        assert_eq!(
            toml_settings,
            Reports {
                template: ReportsTemplate::Inline("lorem ipsum".to_string())
            }
        );
    }
}
