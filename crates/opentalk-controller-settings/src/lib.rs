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

use std::collections::BTreeSet;

use opentalk_types_common::{features::ModuleFeatureId, users::Language};
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

#[derive(Clone, Debug, Deserialize, Default, PartialEq, Eq)]
#[serde(rename_all = "snake_case")]
pub enum ReportsTemplate {
    /// Use the Template included with the application.
    #[default]
    BuiltIn,

    /// Use the Template provided by the user configuration.
    Inline(String),
}

#[derive(Clone, Debug, Deserialize, PartialEq, Eq)]
#[serde(tag = "provider", rename_all = "snake_case")]
pub enum SharedFolder {
    Nextcloud {
        url: url::Url,
        username: String,
        password: String,
        #[serde(default)]
        directory: String,
        #[serde(default)]
        expiry: Option<u64>,
    },
}

#[derive(Clone, Debug, PartialEq, Eq, Deserialize)]
pub struct Avatar {
    #[serde(default = "default_libravatar_url")]
    pub libravatar_url: String,
}

impl Default for Avatar {
    fn default() -> Self {
        Self {
            libravatar_url: default_libravatar_url(),
        }
    }
}

fn default_libravatar_url() -> String {
    "https://seccdn.libravatar.org/avatar/".into()
}

#[derive(Clone, Debug, PartialEq, Eq, Deserialize)]
pub struct CallIn {
    pub tel: String,
    pub enable_phone_mapping: bool,
    pub default_country_code: phonenumber::country::Id,
}

#[derive(Clone, Default, Debug, PartialEq, Eq, Deserialize)]
pub struct Defaults {
    #[serde(default = "default_user_language")]
    pub user_language: Language,
    #[serde(default)]
    pub screen_share_requires_permission: bool,
    #[serde(default)]
    pub disabled_features: BTreeSet<ModuleFeatureId>,
}

fn default_user_language() -> Language {
    "en-US".parse().expect("valid language")
}

#[derive(Clone, Default, Debug, PartialEq, Eq, Deserialize)]
pub struct Endpoints {
    pub disable_users_find: Option<bool>,
    pub users_find_use_kc: Option<bool>,
    #[serde(default)]
    pub event_invite_external_email_address: bool,
    #[serde(default)]
    pub disallow_custom_display_name: bool,
    #[serde(default)]
    pub disable_openapi: bool,
}

#[derive(Clone, Debug, PartialEq, Eq, Deserialize)]
pub struct MinIO {
    pub uri: String,
    pub bucket: String,
    pub access_key: String,
    pub secret_key: String,
}

#[derive(Debug, Default, Clone, PartialEq, Eq, Deserialize)]
pub struct Metrics {
    pub allowlist: Vec<cidr::IpInet>,
}

#[derive(Debug, Clone, PartialEq, Eq, Deserialize)]
#[serde(rename_all = "snake_case", tag = "assignment")]
pub enum TenantAssignment {
    Static {
        static_tenant_id: String,
    },
    ByExternalTenantId {
        #[serde(default = "default_external_tenant_id_user_attribute_name")]
        external_tenant_id_user_attribute_name: String,
    },
}

fn default_external_tenant_id_user_attribute_name() -> String {
    "tenant_id".to_owned()
}

impl Default for TenantAssignment {
    fn default() -> Self {
        Self::Static {
            static_tenant_id: String::from("OpenTalkDefaultTenant"),
        }
    }
}

#[derive(Default, Debug, Clone, PartialEq, Eq, Deserialize)]
pub struct Tenants {
    #[serde(default, flatten)]
    pub assignment: TenantAssignment,
}

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

#[cfg(test)]
mod tests {
    use pretty_assertions::assert_eq;
    use serde_json::json;

    use super::*;

    #[test]
    fn shared_folder_provider_nextcloud() {
        let shared_folder = SharedFolder::Nextcloud {
            url: "https://nextcloud.example.org/".parse().unwrap(),
            username: "exampleuser".to_string(),
            password: "v3rys3cr3t".to_string(),
            directory: "meetings/opentalk".to_string(),
            expiry: Some(34),
        };
        let json = json!({
            "provider": "nextcloud",
            "url": "https://nextcloud.example.org/",
            "username": "exampleuser",
            "password": "v3rys3cr3t",
            "directory": "meetings/opentalk",
            "expiry": 34,
        });

        assert_eq!(
            serde_json::from_value::<SharedFolder>(json).unwrap(),
            shared_folder
        );
    }
}
