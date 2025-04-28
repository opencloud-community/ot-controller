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
mod endpoints;
mod etcd;
mod etherpad;
mod http;
mod http_tls;
mod logging;
mod logging_oltp_tracing;
mod metrics;
mod minio;
mod monitoring;
mod oidc;
mod oidc_and_user_search_builder;
mod oidc_controller;
mod oidc_frontend;
mod rabbitmq;
mod redis;
mod settings;
mod shared_folder;
mod spacedeck;
mod stun;
mod subroom_audio;
mod turn;
mod turn_server;
mod user_search_backend;
mod user_search_backend_keycloak;

pub use authz::Authz;
pub use avatar::Avatar;
pub use call_in::CallIn;
pub use database::Database;
pub use endpoints::Endpoints;
pub use etcd::Etcd;
pub use etherpad::Etherpad;
pub use http::Http;
pub use http_tls::HttpTls;
pub use logging::Logging;
pub use logging_oltp_tracing::LoggingOltpTracing;
pub use metrics::Metrics;
pub use minio::MinIO;
pub use monitoring::Monitoring;
pub use oidc::Oidc;
pub use oidc_controller::OidcController;
pub use oidc_frontend::OidcFrontend;
pub use rabbitmq::RabbitMq;
pub use redis::Redis;
pub use settings::Settings;
pub use shared_folder::SharedFolder;
pub use spacedeck::Spacedeck;
pub use stun::Stun;
pub use subroom_audio::SubroomAudio;
pub use turn::Turn;
pub use turn_server::TurnServer;
pub use user_search_backend::UserSearchBackend;
pub use user_search_backend_keycloak::UserSearchBackendKeycloak;
