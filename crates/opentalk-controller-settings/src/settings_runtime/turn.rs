// SPDX-FileCopyrightText: OpenTalk GmbH <mail@opentalk.eu>
//
// SPDX-License-Identifier: EUPL-1.2

use std::time::Duration;

use super::TurnServer;
use crate::settings_file;

pub const DEFAULT_TURN_LIFETIME: Duration = Duration::from_secs(60);

/// Runtime configuration of TURN servers.
#[derive(Debug, Clone, PartialEq, Eq)]
pub struct Turn {
    /// How long a credential pair should be valid.
    pub lifetime: Duration,

    /// List of configured TURN servers.
    pub servers: Vec<TurnServer>,
}

impl From<settings_file::Turn> for Turn {
    fn from(settings_file::Turn { lifetime, servers }: settings_file::Turn) -> Self {
        Self {
            lifetime: lifetime.unwrap_or(DEFAULT_TURN_LIFETIME),
            servers: servers
                .unwrap_or_default()
                .into_iter()
                .map(Into::into)
                .collect(),
        }
    }
}
