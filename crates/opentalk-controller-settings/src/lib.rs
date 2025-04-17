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
//! is [`SettingsRaw`] (an alias for [`SettingsLoading<OidcAndUserSearchConfiguration>`]) which contains all loaded fields.

use settings_file::{OidcAndUserSearchConfiguration, SettingsLoading};

pub mod settings_file;

mod settings_error;
mod settings_provider;

pub use settings_error::SettingsError;
pub use settings_provider::SettingsProvider;

type Result<T, E = SettingsError> = std::result::Result<T, E>;

pub type SettingsRaw = SettingsLoading<OidcAndUserSearchConfiguration>;
