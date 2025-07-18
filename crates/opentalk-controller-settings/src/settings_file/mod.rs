// SPDX-FileCopyrightText: OpenTalk GmbH <mail@opentalk.eu>
//
// SPDX-License-Identifier: EUPL-1.2

mod authz;
mod avatar;
mod call_in;
mod database;
mod defaults;
mod endpoints;
mod etcd;
mod etherpad;
mod extensions;
mod frontend;
mod http;
mod http_tls;
mod keycloak;
mod live_kit_settings;
mod logging;
mod metrics;
mod minio;
mod monitoring_settings;
mod oidc;
mod oidc_controller;
mod oidc_frontend;
mod operator_information;
mod rabbit_mq_config;
mod redis_config;
mod reports;
mod reports_template;
mod roomserver;
mod settings_raw;
mod shared_folder;
mod spacedeck;
mod subroom_audio;
mod tariff_assignment;
mod tariff_status_mapping;
mod tariffs;
mod tenant_assignment;
mod tenants;
mod user_search;
mod user_search_backend;
mod users_find_behavior;

pub(crate) use authz::Authz;
pub(crate) use avatar::Avatar;
pub(crate) use call_in::CallIn;
pub(crate) use database::Database;
pub(crate) use defaults::Defaults;
pub(crate) use endpoints::Endpoints;
pub(crate) use etcd::Etcd;
pub(crate) use etherpad::Etherpad;
pub(crate) use extensions::Extensions;
pub(crate) use frontend::Frontend;
pub(crate) use http::Http;
pub(crate) use http_tls::HttpTls;
pub(crate) use keycloak::Keycloak;
pub(crate) use live_kit_settings::LiveKitSettings;
pub(crate) use logging::Logging;
pub(crate) use metrics::Metrics;
pub(crate) use minio::MinIO;
pub(crate) use monitoring_settings::MonitoringSettings;
pub(crate) use oidc::Oidc;
pub(crate) use oidc_controller::OidcController;
pub(crate) use oidc_frontend::OidcFrontend;
pub(crate) use operator_information::OperatorInformation;
pub(crate) use rabbit_mq_config::RabbitMqConfig;
pub(crate) use redis_config::RedisConfig;
pub(crate) use reports::Reports;
pub(crate) use reports_template::ReportsTemplate;
pub(crate) use roomserver::RoomServer;
pub use settings_raw::SettingsRaw;
#[cfg(test)]
pub(crate) use settings_raw::{SETTINGS_RAW_MINIMAL_CONFIG_TOML, settings_raw_minimal_example};
pub(crate) use shared_folder::SharedFolder;
pub(crate) use spacedeck::Spacedeck;
pub(crate) use subroom_audio::SubroomAudio;
pub(crate) use tariff_assignment::TariffAssignment;
pub(crate) use tariff_status_mapping::TariffStatusMapping;
pub(crate) use tariffs::Tariffs;
pub(crate) use tenant_assignment::TenantAssignment;
pub(crate) use tenants::Tenants;
pub(crate) use user_search::UserSearch;
pub(crate) use user_search_backend::{UserSearchBackend, UserSearchBackendKeycloakWebapi};
pub use users_find_behavior::UsersFindBehavior;
