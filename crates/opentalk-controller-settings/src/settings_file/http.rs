// SPDX-FileCopyrightText: OpenTalk GmbH <mail@opentalk.eu>
//
// SPDX-License-Identifier: EUPL-1.2

use serde::Deserialize;

use super::HttpTls;

#[derive(Debug, Clone, PartialEq, Eq, Deserialize)]
pub struct Http {
    #[serde(default, skip_serializing_if = "Option::is_none")]
    pub addr: Option<String>,
    #[serde(default = "default_http_port")]
    pub port: u16,
    #[serde(default)]
    pub tls: Option<HttpTls>,
}

impl Default for Http {
    fn default() -> Self {
        Self {
            addr: None,
            port: default_http_port(),
            tls: None,
        }
    }
}

const fn default_http_port() -> u16 {
    11311
}
