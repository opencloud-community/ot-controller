// SPDX-FileCopyrightText: OpenTalk GmbH <mail@opentalk.eu>
//
// SPDX-License-Identifier: EUPL-1.2

//! OpenTalk Controller service
//!
//! This crate contains the default OpenTalk Controller backend implementation.

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

use async_trait::async_trait;
use opentalk_controller_service_facade::OpenTalkControllerServiceBackend;
use opentalk_types_api_v1::auth::{GetLoginResponseBody, OidcProvider};

/// The default [`OpenTalkControllerServiceBackend`] implementation.
#[derive(Debug)]
pub struct ControllerBackend {
    frontend_oidc_provider: OidcProvider,
}

impl ControllerBackend {
    /// Create a new [`ControllerBackend`].
    pub fn new(frontend_oidc_provider: OidcProvider) -> Self {
        Self {
            frontend_oidc_provider,
        }
    }
}

#[async_trait]
impl OpenTalkControllerServiceBackend for ControllerBackend {
    async fn get_login(&self) -> GetLoginResponseBody {
        GetLoginResponseBody {
            oidc: self.frontend_oidc_provider.clone(),
        }
    }
}
