// SPDX-FileCopyrightText: OpenTalk GmbH <mail@opentalk.eu>
//
// SPDX-License-Identifier: EUPL-1.2

use serde::Deserialize;

use super::FrontendOidcConfiguration;
use crate::ControllerOidcConfiguration;

/// OIDC configuration
#[derive(Debug, Clone, PartialEq, Eq, Deserialize)]
pub struct OidcConfiguration {
    pub frontend: FrontendOidcConfiguration,
    pub controller: ControllerOidcConfiguration,
}
