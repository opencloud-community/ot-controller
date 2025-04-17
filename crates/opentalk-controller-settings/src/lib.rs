// SPDX-FileCopyrightText: OpenTalk GmbH <mail@opentalk.eu>
//
// SPDX-License-Identifier: EUPL-1.2

//! Contains the application settings.
//!
//! The application settings are set with a TOML config file. Settings specified in the config file
//! can be overwritten by environment variables. To do so, set an environment variable
//! with the prefix `OPENTALK_CTRL_` followed by the field names you want to set. Nested fields are separated by two underscores `__`.
//! ```sh
//! OPENTALK_CTRL_<field>__<field-of-field>...
//! ```
//!
//! # Example
//!
//! set the `database.url` field:
//! ```sh
//! OPENTALK_CTRL_DATABASE__URL=postgres://postgres:password123@localhost:5432/opentalk
//! ```
//!
//! So the field 'database.max_connections' would resolve to:
//! ```sh
//! OPENTALK_CTRL_DATABASE__MAX_CONNECTIONS=5
//! ```
//!
//! # Note
//!
//! Fields set via environment variables do not affect the underlying config file.
//!
//! # Implementation Details:
//!
//! Setting categories, in which all properties implement a default value, should also implement the [`Default`] trait.
//!
//! [`SettingsLoading<()>`] contains incomplete fields and is an intermediate after
//! loading the raw settings inside [`SettingsProvider::load`]. The final struct with all loaded fields
//! is [`Settings`] (an alias for [`SettingsLoading<OidcAndUserSearchConfiguration>`]) which contains all loaded fields.

use rustc_hash::FxHashSet;
use serde::Deserialize;
use settings_file::{OidcAndUserSearchConfiguration, SettingsLoading};

pub mod settings_file;

mod settings_error;
mod settings_provider;

pub use settings_error::SettingsError;
pub use settings_provider::SettingsProvider;

type Result<T, E = SettingsError> = std::result::Result<T, E>;

pub type Settings = SettingsLoading<OidcAndUserSearchConfiguration>;

#[derive(Debug, Clone, PartialEq, Eq, Deserialize)]
#[serde(rename_all = "snake_case", tag = "assignment")]
pub enum TariffAssignment {
    Static { static_tariff_name: String },
    ByExternalTariffId,
}

impl Default for TariffAssignment {
    fn default() -> Self {
        Self::Static {
            static_tariff_name: String::from("OpenTalkDefaultTariff"),
        }
    }
}

#[derive(Default, Debug, Clone, PartialEq, Eq, Deserialize)]
pub struct TariffStatusMapping {
    pub downgraded_tariff_name: String,
    pub default: FxHashSet<String>,
    pub paid: FxHashSet<String>,
    pub downgraded: FxHashSet<String>,
}

#[derive(Default, Debug, Clone, PartialEq, Eq, Deserialize)]
pub struct Tariffs {
    #[serde(default, flatten)]
    pub assignment: TariffAssignment,

    #[serde(default)]
    pub status_mapping: Option<TariffStatusMapping>,
}

#[derive(Debug, Clone, PartialEq, Eq, Deserialize)]
pub struct LiveKitSettings {
    pub api_key: String,
    pub api_secret: String,
    pub public_url: String,

    // for backwards compatibility
    #[serde(alias = "url")]
    pub service_url: String,
}
