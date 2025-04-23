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

mod database;
mod http;
mod http_tls;
mod oidc;
mod oidc_and_user_search_builder;
mod oidc_controller;
mod oidc_frontend;
mod settings;
mod stun;
mod turn;
mod turn_server;
mod user_search_backend;
mod user_search_backend_keycloak;

pub use database::Database;
pub use http::Http;
pub use http_tls::HttpTls;
pub use oidc::Oidc;
pub use oidc_controller::OidcController;
pub use oidc_frontend::OidcFrontend;
pub use settings::Settings;
pub use stun::Stun;
pub use turn::Turn;
pub use turn_server::TurnServer;
pub use user_search_backend::UserSearchBackend;
pub use user_search_backend_keycloak::UserSearchBackendKeycloak;
