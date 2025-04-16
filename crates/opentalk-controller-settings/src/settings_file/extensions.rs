// SPDX-FileCopyrightText: OpenTalk GmbH <mail@opentalk.eu>
//
// SPDX-License-Identifier: EUPL-1.2

use std::collections::HashMap;

use serde::Deserialize;

#[derive(Default, Debug, Clone, PartialEq, Deserialize)]
pub struct Extensions(pub HashMap<String, config::Value>);

impl Eq for Extensions {}

impl Extensions {
    pub fn contains_key(&self, key: &str) -> bool {
        self.0.contains_key(key)
    }
}
