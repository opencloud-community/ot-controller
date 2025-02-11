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
//! loading the raw settings inside [`SettingsLoading::load`]. The final struct with all loaded fields
//! is [`Settings`] (an alias for [`SettingsLoading<OidcAndUserSearchConfiguration>`]) which contains all loaded fields.

use std::{
    collections::{BTreeSet, HashMap},
    convert::TryFrom,
    net::IpAddr,
    path::PathBuf,
    sync::Arc,
    time::Duration,
};

use arc_swap::ArcSwap;
use config::{Config, Environment, File, FileFormat};
use openidconnect::{ClientId, ClientSecret};
use opentalk_types_common::{features::ModuleFeatureId, users::Language};
use rustc_hash::FxHashSet;
use serde::{Deserialize, Deserializer};
use snafu::{ResultExt, Snafu};
use url::Url;

#[derive(Debug, Snafu)]
pub enum SettingsError {
    #[snafu(display("Failed to read data as config: {}", source), context(false))]
    BuildConfig { source: config::ConfigError },

    #[snafu(display("Failed to apply configuration from {} or environment", file_name))]
    DeserializeConfig {
        file_name: String,
        #[snafu(source(from(serde_path_to_error::Error<config::ConfigError>, Box::new)))]
        source: Box<serde_path_to_error::Error<config::ConfigError>>,
    },

    #[snafu(display("Given base URL is not a base: {}", url))]
    NotBaseUrl { url: Url },

    #[snafu(display("Inconsistent configuration for OIDC and user search, check [keycloak], [endpoints], [oidc] and [user_search] sections"))]
    InconsistentOidcAndUserSearchConfig,
}

type Result<T, E = SettingsError> = std::result::Result<T, E>;

pub type SharedSettings = Arc<ArcSwap<Settings>>;

pub type Settings = SettingsLoading<OidcAndUserSearchConfiguration>;

#[derive(Debug, Clone, Deserialize)]
pub struct SettingsLoading<OIDC> {
    pub database: Database,

    #[serde(default)]
    pub keycloak: Option<Keycloak>,
    #[serde(default)]
    pub oidc: Option<Oidc>,
    #[serde(default)]
    pub user_search: Option<UserSearch>,

    /// The OIDC and user search configuration.
    ///
    /// This configuration is built from the [`Self::oidc`] and [`Self::user_search`]
    /// fields or from the legacy [`Self::keycloak`] field.
    #[serde(skip)]
    pub oidc_and_user_search: OIDC,

    pub http: Http,
    #[serde(default)]
    pub turn: Option<Turn>,
    #[serde(default)]
    pub stun: Option<Stun>,
    #[serde(default)]
    pub redis: Option<RedisConfig>,
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
    pub etcd: Option<Etcd>,

    #[serde(default)]
    pub etherpad: Option<Etherpad>,

    #[serde(default)]
    pub spacedeck: Option<Spacedeck>,

    #[serde(default)]
    pub subroom_audio: Option<SubroomAudio>,

    #[serde(default)]
    pub reports: Option<Reports>,

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
    pub monitoring: Option<MonitoringSettings>,

    #[serde(default)]
    pub tenants: Tenants,

    #[serde(default)]
    pub tariffs: Tariffs,

    pub livekit: LiveKitSettings,

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

#[derive(Debug, Clone, Deserialize)]
pub struct MonitoringSettings {
    #[serde(default = "default_monitoring_port")]
    pub port: u16,
    #[serde(default = "default_monitoring_addr")]
    pub addr: IpAddr,
}

fn default_monitoring_port() -> u16 {
    11411
}

fn default_monitoring_addr() -> IpAddr {
    [0, 0, 0, 0].into()
}

/// OIDC and user search configuration
#[derive(Debug, Clone, Deserialize)]
pub struct OidcAndUserSearchConfiguration {
    pub oidc: OidcConfiguration,
    pub user_search: UserSearchConfiguration,
}

/// OIDC configuration
#[derive(Debug, Clone, Deserialize)]
pub struct OidcConfiguration {
    pub frontend: FrontendOidcConfiguration,
    pub controller: ControllerOidcConfiguration,
}

/// OIDC configuration for frontend
#[derive(Debug, Clone, Deserialize)]
pub struct FrontendOidcConfiguration {
    pub auth_base_url: Url,
    pub client_id: ClientId,
}

/// OIDC configuration for controller
#[derive(Debug, Clone, Deserialize)]
pub struct ControllerOidcConfiguration {
    pub auth_base_url: Url,
    pub client_id: ClientId,
    pub client_secret: ClientSecret,
}

/// User search configuration
#[derive(Debug, Clone, Deserialize)]
pub struct UserSearchConfiguration {
    pub backend: UserSearchBackend,
    pub api_base_url: Url,
    pub client_id: ClientId,
    pub client_secret: ClientSecret,
    pub external_id_user_attribute_name: Option<String>,
    pub users_find_behavior: UsersFindBehavior,
}

impl<OIDC> SettingsLoading<OIDC> {
    /// internal url builder
    fn build_url<I>(base_url: Url, path_segments: I) -> Result<Url>
    where
        I: IntoIterator,
        I::Item: AsRef<str>,
    {
        let err_url = base_url.clone();
        let mut url = base_url;
        url.path_segments_mut()
            .map_err(|_| SettingsError::NotBaseUrl { url: err_url })?
            .extend(path_segments);
        Ok(url)
    }

    /// Builds the effective OIDC and user search configuration, either from the deprecated `[keycloak]` section
    /// and some deprecated `[endpoints]` settings or from the new `[oidc]` and `[user_search]` sections.
    fn build_oidc_and_user_search_configuration(&self) -> Result<OidcAndUserSearchConfiguration> {
        let keycloak = self.keycloak.clone();
        let disable_users_find = self.endpoints.disable_users_find;
        let users_find_use_kc = self.endpoints.users_find_use_kc;
        let oidc = self.oidc.clone();
        let user_search = self.user_search.clone();

        match (
            keycloak,
            disable_users_find,
            users_find_use_kc,
            oidc,
            user_search,
        ) {
            // Only the new OIDC and user search configuration is present
            (None, None, None, Some(oidc), Some(user_search)) => {
                Self::build_new_oidc_and_user_search_configuration(oidc, user_search)
            }
            // Only the legacy OIDC and user search configuration is present
            (Some(keycloak), _, _, None, None) => {
                Self::build_legacy_oidc_and_user_search_configuration(
                    &keycloak,
                    disable_users_find,
                    users_find_use_kc,
                )?
            }
            // The OIDC and user search configuration is inconsistent
            _ => Err(SettingsError::InconsistentOidcAndUserSearchConfig),
        }
    }

    /// Builds the effective OIDC and user search configuration from the new `[oidc]` and `[user_search]` sections.
    fn build_new_oidc_and_user_search_configuration(
        oidc: Oidc,
        user_search: UserSearch,
    ) -> Result<OidcAndUserSearchConfiguration, SettingsError> {
        // Frontend-specific OIDC configuration
        let frontend_auth_base_url = oidc.frontend.authority.unwrap_or(oidc.authority.clone());
        let frontend_client_id = oidc.frontend.client_id.clone();

        // Controller-specific OIDC configuration
        let controller_auth_base_url = oidc.controller.authority.unwrap_or(oidc.authority);
        let controller_client_id = oidc.controller.client_id.clone();
        let controller_client_secret = oidc.controller.client_secret.clone();

        // User search configuration
        let backend = user_search.backend;
        let api_base_url = user_search.api_base_url;
        let user_search_client_id = user_search
            .client_id
            .unwrap_or(controller_client_id.clone());
        let user_search_client_secret = user_search
            .client_secret
            .unwrap_or(controller_client_secret.clone());
        let external_id_user_attribute_name = user_search.external_id_user_attribute_name.clone();
        let users_find_behavior = user_search.users_find_behavior;

        // Assemble the entire effective OIDC and user search configuration
        let frontend = FrontendOidcConfiguration {
            auth_base_url: frontend_auth_base_url,
            client_id: frontend_client_id,
        };
        let controller = ControllerOidcConfiguration {
            auth_base_url: controller_auth_base_url,
            client_id: controller_client_id,
            client_secret: controller_client_secret.clone(),
        };
        let oidc = OidcConfiguration {
            frontend,
            controller,
        };
        let api = UserSearchConfiguration {
            backend,
            api_base_url,
            client_id: user_search_client_id,
            client_secret: user_search_client_secret,
            external_id_user_attribute_name,
            users_find_behavior,
        };
        Ok(OidcAndUserSearchConfiguration {
            oidc,
            user_search: api,
        })
    }

    /// Builds the effective OIDC and user search configuration from the deprecated `[keycloak]` section
    /// and some deprecated `[endpoints]` settings.
    fn build_legacy_oidc_and_user_search_configuration(
        keycloak: &Keycloak,
        disable_users_find: Option<bool>,
        users_find_use_kc: Option<bool>,
    ) -> Result<Result<OidcAndUserSearchConfiguration, SettingsError>, SettingsError> {
        log::warn!(
                    "You are using deprecated OIDC and user search settings. See docs for [oidc] and [user_search] configuration sections."
                );
        // Collect legacy OIDC and user search settings
        let backend = UserSearchBackend::KeycloakWebapi;
        let api_base_url = Self::build_url(
            keycloak.base_url.clone(),
            ["admin", "realms", &keycloak.realm],
        )?;
        let auth_base_url =
            Self::build_url(keycloak.base_url.clone(), ["realms", &keycloak.realm])?;
        let client_id = keycloak.client_id.clone();
        let client_secret = keycloak.client_secret.clone();
        let external_id_user_attribute_name = keycloak.external_id_user_attribute_name.clone();
        let users_find_behavior = match (
            disable_users_find.unwrap_or_default(),
            users_find_use_kc.unwrap_or_default(),
        ) {
            (true, _) => UsersFindBehavior::Disabled,
            (false, false) => UsersFindBehavior::FromDatabase,
            (false, true) => UsersFindBehavior::FromUserSearchBackend,
        };

        // Assemble the entire effective OIDC and user search configuration
        let frontend = FrontendOidcConfiguration {
            auth_base_url: auth_base_url.clone(),
            client_id: client_id.clone(),
        };
        let controller = ControllerOidcConfiguration {
            auth_base_url,
            client_id: client_id.clone(),
            client_secret: client_secret.clone().clone(),
        };
        let oidc = OidcConfiguration {
            frontend,
            controller,
        };
        let api = UserSearchConfiguration {
            backend,
            api_base_url,
            client_id,
            client_secret,
            external_id_user_attribute_name,
            users_find_behavior,
        };
        Ok(Ok(OidcAndUserSearchConfiguration {
            oidc,
            user_search: api,
        }))
    }

    /// Creates a new Settings instance from the provided TOML file.
    /// Specific fields can be set or overwritten with environment variables (See struct level docs for more details).
    pub fn load(file_name: &str) -> Result<Settings> {
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

        let this: SettingsLoading<()> =
            serde_path_to_error::deserialize(config).context(DeserializeConfigSnafu {
                file_name: file_name.to_owned(),
            })?;

        let oidc_and_user_search = this.build_oidc_and_user_search_configuration()?;

        Ok(Settings {
            oidc_and_user_search,
            database: this.database,
            keycloak: this.keycloak,
            oidc: this.oidc,
            user_search: this.user_search,
            http: this.http,
            turn: this.turn,
            stun: this.stun,
            redis: this.redis,
            rabbit_mq: this.rabbit_mq,
            logging: this.logging,
            authz: this.authz,
            avatar: this.avatar,
            metrics: this.metrics,
            etcd: this.etcd,
            etherpad: this.etherpad,
            spacedeck: this.spacedeck,
            reports: this.reports,
            subroom_audio: this.subroom_audio,
            shared_folder: this.shared_folder,
            call_in: this.call_in,
            defaults: this.defaults,
            endpoints: this.endpoints,
            minio: this.minio,
            monitoring: this.monitoring,
            tenants: this.tenants,
            tariffs: this.tariffs,
            livekit: this.livekit,
            extensions: this.extensions,
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
pub struct Oidc {
    pub authority: Url,
    pub frontend: OidcFrontend,
    pub controller: OidcController,
}

#[derive(Debug, Clone, Deserialize)]
pub struct OidcFrontend {
    pub authority: Option<Url>,
    pub client_id: ClientId,
}

#[derive(Debug, Clone, Deserialize)]
pub struct OidcController {
    pub authority: Option<Url>,
    pub client_id: ClientId,
    pub client_secret: ClientSecret,
}

#[derive(Debug, Clone, Deserialize)]
pub struct UserSearch {
    #[serde(flatten)]
    pub backend: UserSearchBackend,
    pub api_base_url: Url,
    pub client_id: Option<ClientId>,
    pub client_secret: Option<ClientSecret>,
    pub external_id_user_attribute_name: Option<String>,
    #[serde(flatten)]
    pub users_find_behavior: UsersFindBehavior,
}

#[derive(Clone, Debug, Deserialize, PartialEq, Eq)]
#[serde(rename_all = "snake_case", tag = "backend")]
pub enum UserSearchBackend {
    KeycloakWebapi,
}

#[derive(Clone, Debug, Deserialize, PartialEq, Eq)]
#[serde(rename_all = "snake_case", tag = "users_find_behavior")]
pub enum UsersFindBehavior {
    Disabled,
    FromDatabase,
    FromUserSearchBackend,
}

#[derive(Debug, Clone, Deserialize)]
pub struct Http {
    #[serde(default, skip_serializing_if = "Option::is_none")]
    pub addr: Option<String>,
    #[serde(default = "default_http_port")]
    pub port: u16,
    #[serde(default)]
    pub tls: Option<HttpTls>,
}

impl Default for Http {
    fn default() -> Self {
        Self {
            addr: None,
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

#[derive(Clone, Debug, Deserialize)]
pub struct Etcd {
    pub urls: Vec<url::Url>,
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

#[derive(Clone, Debug, Deserialize)]
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

#[derive(Clone, Default, Debug, Deserialize)]
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

#[derive(Clone, Default, Debug, Deserialize)]
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

#[derive(Debug, Clone, Deserialize)]
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
    use std::env;

    use pretty_assertions::assert_eq;
    use serde_json::json;

    use super::*;

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
