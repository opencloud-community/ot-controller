// SPDX-FileCopyrightText: OpenTalk GmbH <mail@opentalk.eu>
//
// SPDX-License-Identifier: EUPL-1.2

//! Handles the application settings via a config file and environment variables.
use std::sync::Arc;

use actix_web::web::Data;
use arc_swap::ArcSwap;
pub use opentalk_controller_settings::*;
use snafu::ResultExt;

use crate::{cli::Args, Result};

pub type SharedSettingsActix = Data<ArcSwap<Settings>>;

/// Reload the settings from the `config_path` & the environment
///
/// Not all settings are used, as most of the settings are not reloadable while the
/// controller is running.
pub(crate) fn reload_settings(shared_settings: SharedSettings, config_path: &str) -> Result<()> {
    let new_settings = Settings::load(config_path).whatever_context("Failed to load settings")?;
    let mut current_settings = (*shared_settings.load_full()).clone();

    // reload extensions config
    current_settings.extensions = new_settings.extensions;

    // reload turn settings
    current_settings.turn = new_settings.turn;

    // reload metrics
    current_settings.metrics = new_settings.metrics;

    // reload avatar
    current_settings.avatar = new_settings.avatar;

    // reload call in
    current_settings.call_in = new_settings.call_in;

    // replace the shared settings with the modified ones
    shared_settings.store(Arc::new(current_settings));

    Ok(())
}

/// Loads settings from program arguments and config file
///
/// The settings specified in the CLI-Arguments have a higher priority than the settings specified in the config file
pub fn load_settings(args: &Args) -> Result<Settings, SettingsError> {
    Settings::load(&args.config)
}

#[cfg(test)]
mod tests {
    use opentalk_controller_settings::{Settings, SettingsError};

    #[test]
    fn example_toml() -> Result<(), SettingsError> {
        Settings::load("../../extra/example.toml")?;
        Ok(())
    }
}
