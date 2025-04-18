// SPDX-FileCopyrightText: OpenTalk GmbH <mail@opentalk.eu>
//
// SPDX-License-Identifier: EUPL-1.2

use serde::Deserialize;

#[derive(Debug, Clone, PartialEq, Eq, Deserialize)]
pub struct Database {
    pub url: String,
    #[serde(default = "default_max_connections")]
    pub max_connections: u32,
}

pub(super) fn default_max_connections() -> u32 {
    100
}
