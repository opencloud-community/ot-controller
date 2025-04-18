// SPDX-FileCopyrightText: OpenTalk GmbH <mail@opentalk.eu>
//
// SPDX-License-Identifier: EUPL-1.2

use serde::Deserialize;

#[derive(Default, Clone, Debug, Deserialize, PartialEq, Eq)]
#[serde(rename_all = "snake_case", tag = "users_find_behavior")]
pub enum UsersFindBehavior {
    #[default]
    Disabled,
    FromDatabase,
    FromUserSearchBackend,
}
