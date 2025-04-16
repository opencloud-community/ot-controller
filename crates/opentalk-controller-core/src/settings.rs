// SPDX-FileCopyrightText: OpenTalk GmbH <mail@opentalk.eu>
//
// SPDX-License-Identifier: EUPL-1.2

//! Handles the application settings via a config file and environment variables.
pub use opentalk_controller_settings::*;

use crate::{cli::Args, Result};

/// Loads settings from program arguments and config file
///
/// The settings specified in the CLI-Arguments have a higher priority than the settings specified in the config file
pub fn load_settings(args: &Args) -> Result<SettingsProvider, SettingsError> {
    SettingsProvider::load(&args.config)
}

#[cfg(test)]
mod tests {
    use opentalk_controller_settings::{SettingsError, SettingsProvider};

    #[test]
    fn example_toml() -> Result<(), SettingsError> {
        SettingsProvider::load("../../extra/example.toml")?;
        Ok(())
    }
}
