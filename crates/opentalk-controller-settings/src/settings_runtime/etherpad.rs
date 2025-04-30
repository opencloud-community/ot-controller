// SPDX-FileCopyrightText: OpenTalk GmbH <mail@opentalk.eu>
//
// SPDX-License-Identifier: EUPL-1.2

use url::Url;

use crate::settings_file;

/// Etherpad settings.
#[derive(Debug, Clone, PartialEq, Eq)]
pub struct Etherpad {
    /// The Etherpad instance url.
    pub url: Url,

    /// The API key.
    pub api_key: String,
}

impl From<settings_file::Etherpad> for Etherpad {
    fn from(settings_file::Etherpad { url, api_key }: settings_file::Etherpad) -> Self {
        Self { url, api_key }
    }
}
