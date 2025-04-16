// SPDX-FileCopyrightText: OpenTalk GmbH <mail@opentalk.eu>
//
// SPDX-License-Identifier: EUPL-1.2

use openidconnect::{ClientId, ClientSecret};
use serde::Deserialize;
use url::Url;

use crate::{UserSearchBackend, UsersFindBehavior};

#[derive(Debug, Clone, Deserialize)]
pub struct UserSearch {
    #[serde(flatten)]
    pub backend: UserSearchBackend,
    pub api_base_url: Url,
    pub client_id: Option<ClientId>,
    pub client_secret: Option<ClientSecret>,
    pub external_id_user_attribute_name: Option<String>,
    #[serde(flatten)]
    pub users_find_behavior: UsersFindBehavior,
}

impl PartialEq for UserSearch {
    fn eq(&self, other: &Self) -> bool {
        self.backend.eq(&other.backend)
            && self.api_base_url.eq(&other.api_base_url)
            && self.client_id.eq(&other.client_id)
            && self
                .client_secret
                .as_ref()
                .map(|s| s.secret())
                .eq(&other.client_secret.as_ref().map(|s| s.secret()))
            && self
                .external_id_user_attribute_name
                .eq(&other.external_id_user_attribute_name)
            && self.users_find_behavior.eq(&other.users_find_behavior)
    }
}

impl Eq for UserSearch {}
