// SPDX-FileCopyrightText: OpenTalk GmbH <mail@opentalk.eu>
//
// SPDX-License-Identifier: EUPL-1.2

use serde::Deserialize;

#[derive(Debug, Clone, PartialEq, Eq, Deserialize)]
pub(crate) struct LiveKitSettings {
    pub api_key: String,
    pub api_secret: String,
    pub public_url: String,

    // for backwards compatibility
    #[serde(alias = "url")]
    pub service_url: String,
}
