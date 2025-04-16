// SPDX-FileCopyrightText: OpenTalk GmbH <mail@opentalk.eu>
//
// SPDX-License-Identifier: EUPL-1.2

use serde::Deserialize;
use url::Url;

use crate::{OidcController, OidcFrontend};

#[derive(Debug, Clone, PartialEq, Eq, Deserialize)]
pub struct Oidc {
    pub authority: Url,
    pub frontend: OidcFrontend,
    pub controller: OidcController,
}
