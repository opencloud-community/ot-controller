// SPDX-FileCopyrightText: OpenTalk GmbH <mail@opentalk.eu>
//
// SPDX-License-Identifier: EUPL-1.2

use openidconnect::ClientId;
use serde::Deserialize;
use url::Url;

/// OIDC configuration for frontend
#[derive(Debug, Clone, PartialEq, Eq, Deserialize)]
pub struct FrontendOidcConfiguration {
    pub auth_base_url: Url,
    pub client_id: ClientId,
}
