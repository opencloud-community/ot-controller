// SPDX-FileCopyrightText: OpenTalk GmbH <mail@opentalk.eu>
//
// SPDX-License-Identifier: EUPL-1.2

#![deny(
    bad_style,
    missing_debug_implementations,
    missing_docs,
    overflowing_literals,
    patterns_in_fns_without_body,
    trivial_casts,
    trivial_numeric_casts,
    unsafe_code,
    unused,
    unused_extern_crates,
    unused_import_braces,
    unused_qualifications,
    unused_results
)]

mod authz;
mod avatar;
mod call_in;
mod database;
mod defaults;
mod endpoints;
mod etcd;
mod etherpad;
mod frontend;
mod http;
mod http_tls;
mod livekit;
mod logging;
mod logging_oltp_tracing;
mod metrics;
mod minio;
mod monitoring;
mod oidc;
mod oidc_and_user_search_builder;
mod oidc_controller;
mod oidc_frontend;
mod operator_information;
mod rabbitmq;
mod redis;
mod roomserver;
pub(crate) mod settings;
mod shared_folder;
mod spacedeck;
mod subroom_audio;
mod tariff_assignment;
mod tariff_status_mapping;
mod tariffs;
mod tenant_assignment;
mod tenants;
mod user_search_backend;
mod user_search_backend_keycloak;

pub use authz::Authz;
pub use avatar::{Avatar, DEFAULT_LIBRAVATAR_URL};
pub use call_in::CallIn;
pub use database::Database;
pub use defaults::Defaults;
pub use endpoints::Endpoints;
pub use etcd::Etcd;
pub use etherpad::Etherpad;
pub use frontend::Frontend;
pub use http::Http;
pub use http_tls::HttpTls;
pub use livekit::LiveKit;
pub use logging::Logging;
pub use logging_oltp_tracing::LoggingOltpTracing;
pub use metrics::Metrics;
pub use minio::MinIO;
pub use monitoring::Monitoring;
pub use oidc::Oidc;
pub use oidc_controller::OidcController;
pub use oidc_frontend::OidcFrontend;
pub use operator_information::OperatorInformation;
pub use rabbitmq::RabbitMq;
pub use redis::Redis;
pub use roomserver::RoomServer;
pub use settings::Settings;
pub use shared_folder::SharedFolder;
pub use spacedeck::Spacedeck;
pub use subroom_audio::SubroomAudio;
pub use tariff_assignment::{DEFAULT_STATIC_TARIFF_NAME, TariffAssignment};
pub use tariff_status_mapping::TariffStatusMapping;
pub use tariffs::Tariffs;
pub use tenant_assignment::{
    DEFAULT_EXTERNAL_TENANT_ID_USER_ATTRIBUTE_NAME, DEFAULT_STATIC_TENANT_ID, TenantAssignment,
};
pub use tenants::Tenants;
pub use user_search_backend::UserSearchBackend;
pub use user_search_backend_keycloak::UserSearchBackendKeycloak;
