// SPDX-FileCopyrightText: OpenTalk GmbH <mail@opentalk.eu>
//
// SPDX-License-Identifier: EUPL-1.2

use serde::Deserialize;

#[derive(Clone, Default, Debug, PartialEq, Eq, Deserialize)]
pub struct Endpoints {
    pub disable_users_find: Option<bool>,
    pub users_find_use_kc: Option<bool>,
    #[serde(default)]
    pub event_invite_external_email_address: bool,
    #[serde(default)]
    pub disallow_custom_display_name: bool,
    #[serde(default)]
    pub disable_openapi: bool,
}
