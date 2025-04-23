// SPDX-FileCopyrightText: OpenTalk GmbH <mail@opentalk.eu>
//
// SPDX-License-Identifier: EUPL-1.2

use crate::settings_file;

/// Runtime configuration of TURN servers.
#[derive(Debug, Clone, PartialEq, Eq)]
pub struct Stun {
    /// STUN URIs following RFC7065.
    pub uris: Vec<String>,
}

impl From<settings_file::Stun> for Stun {
    fn from(settings_file::Stun { uris }: settings_file::Stun) -> Self {
        Self { uris }
    }
}
