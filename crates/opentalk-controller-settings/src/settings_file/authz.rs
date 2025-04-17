// SPDX-FileCopyrightText: OpenTalk GmbH <mail@opentalk.eu>
//
// SPDX-License-Identifier: EUPL-1.2

use serde::Deserialize;

#[derive(Clone, Debug, PartialEq, Eq, Deserialize)]
pub struct Authz {
    #[serde(default = "authz_default_synchronize_controller")]
    pub synchronize_controllers: bool,
}

impl Default for Authz {
    fn default() -> Self {
        Self {
            synchronize_controllers: authz_default_synchronize_controller(),
        }
    }
}

fn authz_default_synchronize_controller() -> bool {
    true
}
