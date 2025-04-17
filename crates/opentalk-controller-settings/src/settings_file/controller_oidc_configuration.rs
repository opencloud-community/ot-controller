// SPDX-FileCopyrightText: OpenTalk GmbH <mail@opentalk.eu>
//
// SPDX-License-Identifier: EUPL-1.2

use openidconnect::{ClientId, ClientSecret};
use serde::Deserialize;
use url::Url;

/// OIDC configuration for controller
#[derive(Debug, Clone, Deserialize)]
pub struct ControllerOidcConfiguration {
    pub auth_base_url: Url,
    pub client_id: ClientId,
    pub client_secret: ClientSecret,
}

impl PartialEq for ControllerOidcConfiguration {
    fn eq(&self, other: &Self) -> bool {
        self.auth_base_url.eq(&other.auth_base_url)
            && self.client_id.eq(&other.client_id)
            && self.client_secret.secret().eq(other.client_secret.secret())
    }
}

impl Eq for ControllerOidcConfiguration {}
