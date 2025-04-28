// SPDX-FileCopyrightText: OpenTalk GmbH <mail@opentalk.eu>
//
// SPDX-License-Identifier: EUPL-1.2

use url::Url;

use crate::settings_file;

/// Spacedeck settings.
#[derive(Debug, Clone, PartialEq, Eq)]
pub struct Spacedeck {
    /// The Spacedeck instance url.
    pub url: Url,

    /// The API key.
    pub api_key: String,
}

impl From<settings_file::Spacedeck> for Spacedeck {
    fn from(settings_file::Spacedeck { url, api_key }: settings_file::Spacedeck) -> Self {
        Self { url, api_key }
    }
}
