// SPDX-FileCopyrightText: OpenTalk GmbH <mail@opentalk.eu>
//
// SPDX-License-Identifier: EUPL-1.2

use serde::Deserialize;

use super::HttpTls;

#[derive(Default, Debug, Clone, PartialEq, Eq, Deserialize)]
pub struct Http {
    #[serde(default, skip_serializing_if = "Option::is_none")]
    pub addr: Option<String>,

    #[serde(default, skip_serializing_if = "Option::is_none")]
    pub port: Option<u16>,

    #[serde(default, skip_serializing_if = "Option::is_none")]
    pub tls: Option<HttpTls>,
}
