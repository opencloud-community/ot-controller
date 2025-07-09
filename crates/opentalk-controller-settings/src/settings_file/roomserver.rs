// SPDX-FileCopyrightText: OpenTalk GmbH <mail@opentalk.eu>
//
// SPDX-License-Identifier: EUPL-1.2

use serde::Deserialize;
use url::Url;

#[derive(Clone, Debug, Deserialize, PartialEq, Eq)]
pub struct RoomServer {
    pub url: Url,

    pub api_token: String,
}
