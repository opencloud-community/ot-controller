// SPDX-FileCopyrightText: OpenTalk GmbH <mail@opentalk.eu>
//
// SPDX-License-Identifier: EUPL-1.2

use serde::Deserialize;

use super::{OidcConfiguration, UserSearchConfiguration};

/// OIDC and user search configuration
#[derive(Debug, Clone, PartialEq, Eq, Deserialize)]
pub struct OidcAndUserSearchConfiguration {
    pub oidc: OidcConfiguration,
    pub user_search: UserSearchConfiguration,
}
