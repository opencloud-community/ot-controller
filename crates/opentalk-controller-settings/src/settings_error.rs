// SPDX-FileCopyrightText: OpenTalk GmbH <mail@opentalk.eu>
//
// SPDX-License-Identifier: EUPL-1.2

use std::path::PathBuf;

use snafu::Snafu;
use url::Url;

#[derive(Debug, Snafu)]
#[snafu(visibility(pub(crate)))]
pub enum SettingsError {
    #[snafu(display("Failed to read data as config: {}", source), context(false))]
    BuildConfig { source: config::ConfigError },

    #[snafu(display("Failed to apply configuration from {} or environment", file_path.to_string_lossy()))]
    DeserializeConfig {
        file_path: PathBuf,

        #[snafu(source(from(serde_path_to_error::Error<config::ConfigError>, Box::new)))]
        source: Box<serde_path_to_error::Error<config::ConfigError>>,
    },

    #[snafu(display("Given base URL is not a base: {}", url))]
    NotBaseUrl { url: Url },

    #[snafu(display("Inconsistent configuration for OIDC and user search, check [keycloak], [endpoints], [oidc] and [user_search] sections"))]
    InconsistentOidcAndUserSearchConfig,

    #[snafu(display("Found a {conflicting_field} configuration value which is not allowed when a [oidc] section is configured"))]
    OidcInvalidConfiguration { conflicting_field: &'static str },

    #[snafu(display("Missing OIDC configuration. Either an [oidc] or a deprecated [keycloak] section must be present in the configuration"))]
    OidcConfigurationMissing,
}
