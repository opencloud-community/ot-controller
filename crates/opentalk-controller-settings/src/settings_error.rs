// SPDX-FileCopyrightText: OpenTalk GmbH <mail@opentalk.eu>
//
// SPDX-License-Identifier: EUPL-1.2

use snafu::Snafu;
use url::Url;

#[derive(Debug, Snafu)]
#[snafu(visibility(pub(crate)))]
pub enum SettingsError {
    #[snafu(display("Failed to read data as config: {}", source), context(false))]
    BuildConfig { source: config::ConfigError },

    #[snafu(display("Failed to apply configuration from {} or environment", file_name))]
    DeserializeConfig {
        file_name: String,

        #[snafu(source(from(serde_path_to_error::Error<config::ConfigError>, Box::new)))]
        source: Box<serde_path_to_error::Error<config::ConfigError>>,
    },

    #[snafu(display("Given base URL is not a base: {}", url))]
    NotBaseUrl { url: Url },

    #[snafu(display("Inconsistent configuration for OIDC and user search, check [keycloak], [endpoints], [oidc] and [user_search] sections"))]
    InconsistentOidcAndUserSearchConfig,
}
