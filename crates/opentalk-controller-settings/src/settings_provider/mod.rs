// SPDX-FileCopyrightText: OpenTalk GmbH <mail@opentalk.eu>
//
// SPDX-License-Identifier: EUPL-1.2

use std::sync::Arc;

use arc_swap::ArcSwap;

use crate::{Result, Settings, SettingsRaw};

mod loading;

/// A struct for loading and holding the runtime settings.
#[derive(Debug, Clone)]
pub struct SettingsProvider {
    settings: Arc<ArcSwap<Settings>>,
}

impl SettingsProvider {
    /// Load the settings from a TOML file.
    ///
    /// This will succeed in case the file could be loaded successfully.
    /// Environment variables in the `OPENTALK_CTRL_*` pattern are considiered
    /// and will override the settings found in the file.
    pub fn load(file_name: &str) -> Result<Self> {
        let settings_raw = Self::load_raw(file_name)?;
        Self::new_raw(settings_raw)
    }

    /// Create a new [`SettingsProvider`] with settings that are already loaded.
    pub fn new_raw(settings_raw: SettingsRaw) -> Result<Self> {
        let settings = Settings::try_from(settings_raw)?;
        Ok(Self {
            settings: Arc::new(ArcSwap::new(Arc::new(settings))),
        })
    }

    /// Get an `[Arc]` holding the current raw settings.
    ///
    /// The returned settings will remain unchanged even if the settings are
    /// reloaded by the [`SettingsProvider`]. A new [`Arc`] will be created
    /// internally by the `reload` function. This allows consistent use of a
    /// "snapshot" inside a function by calling `get_raw` once, and then using
    /// the returned value.
    pub fn get(&self) -> Arc<Settings> {
        self.settings.load_full().clone()
    }

    /// Reload the settings from a TOML file.
    ///
    /// Not all settings are used, as most of the settings are not reloadable
    /// while the controller is running.
    ///
    /// This will succeed in case the file could be loaded successfully.
    /// Environment variables in the `OPENTALK_CTRL_*` pattern are considiered
    /// and will override the settings found in the file.
    ///
    /// If loading the settings fails, an error is returned from this function
    /// and stored configuration will remain unchanged.
    ///
    /// Any "snapshots" handed out to callers by the `get` function remain
    /// unchanged, so wherever these are used, the values will not change.
    /// Because an `Arc` was given to these callers, the value will be freed
    /// once the last reference to it has been dropped.
    pub fn reload(&self, config_path: &str) -> Result<()> {
        let settings_raw = Self::load_raw(config_path)?;

        let mut current_settings = (*self.settings.load_full()).clone();

        current_settings.try_reload_from(settings_raw)?;

        // replace the shared settings with the modified ones
        self.settings.store(Arc::new(current_settings));

        Ok(())
    }
}

#[cfg(test)]
mod tests {
    use std::{env, fs::File, io::Write as _};

    use pretty_assertions::{assert_eq, assert_matches, assert_ne};
    use tempfile::tempdir;

    use super::SettingsProvider;
    use crate::{
        settings_file::SETTINGS_RAW_MINIMAL_CONFIG_TOML,
        settings_runtime::settings::minimal_example, SettingsError,
    };

    #[test]
    fn load_minimal() {
        let tempdir = tempdir().unwrap();

        let path = tempdir.path().join("controller.toml");

        {
            let mut file = File::create(&path).unwrap();
            writeln!(file, "{SETTINGS_RAW_MINIMAL_CONFIG_TOML}")
                .expect("temp file should be writable");
        }

        let settings_provider =
            SettingsProvider::load(path.to_str().expect("valid file path expected"))
                .expect("valid configuration expected");

        assert_eq!(&(*settings_provider.get()), &minimal_example());
    }

    #[test]
    fn load_invalid() {
        let tempdir = tempdir().unwrap();

        let path = tempdir.path().join("controller.toml");

        {
            // Create an empty file which will result in an invalid definition
            let _file = File::create(&path).unwrap();
        }

        assert_matches!(
            SettingsProvider::load(path.to_str().expect("valid file path expected")),
            Err(SettingsError::DeserializeConfig {
                file_name: _,
                source: _
            })
        );
    }

    #[test]
    fn reload() {
        env::remove_var("OPENTALK_CTRL_DATABASE__URL");
        env::remove_var("OPENTALK_CTRL_HTTP__PORT");
        env::remove_var("OPENTALK_CTRL_HTTP__DEFAULTS__USER_LANGUAGE");
        env::remove_var("OPENTALK_CTRL_HTTP__DEFAULTS__SCREEN_SHARE_REQUIRES_PERMISSION");

        let tempdir = tempdir().unwrap();

        let modified_path = tempdir.path().join("controller_modified.toml");
        let minimal_path = tempdir.path().join("controller_minimal.toml");

        {
            let mut file = File::create(&modified_path).unwrap();
            writeln!(
                file,
                r#"
                {SETTINGS_RAW_MINIMAL_CONFIG_TOML}

                [call_in]
                tel = "+55667788"
                enable_phone_mapping = true
                default_country_code = "DE"
                "#
            )
            .expect("temp file should be writable");
        }
        {
            let mut file = File::create(&minimal_path).unwrap();
            writeln!(file, "{SETTINGS_RAW_MINIMAL_CONFIG_TOML}")
                .expect("temp file should be writable");
        }

        let settings_provider =
            SettingsProvider::load(modified_path.to_str().expect("valid file path expected"))
                .expect("valid configuration expected");

        assert_ne!(&(*settings_provider.get()), &minimal_example());

        settings_provider
            .reload(minimal_path.to_str().expect("valid file path expected"))
            .expect("reload is expected to succeed");

        assert_eq!(&(*settings_provider.get()), &minimal_example());
    }

    #[test]
    fn reload_invalid() {
        let tempdir = tempdir().unwrap();

        let invalid_path = tempdir.path().join("controller_invalid.toml");
        let minimal_path = tempdir.path().join("controller_minimal.toml");

        {
            // Create an empty file which will result in an invalid definition
            let _file = File::create(&invalid_path).unwrap();
        }
        {
            let mut file = File::create(&minimal_path).unwrap();
            writeln!(file, "{SETTINGS_RAW_MINIMAL_CONFIG_TOML}")
                .expect("temp file should be writable");
        }

        let settings_provider =
            SettingsProvider::load(minimal_path.to_str().expect("valid file path expected"))
                .expect("valid configuration expected");

        assert_eq!(&(*settings_provider.get()), &minimal_example());

        assert_matches!(
            settings_provider.reload(invalid_path.to_str().expect("valid file path expected")),
            Err(SettingsError::DeserializeConfig {
                file_name: _,
                source: _
            })
        );

        assert_eq!(&(*settings_provider.get()), &minimal_example());
    }
}
