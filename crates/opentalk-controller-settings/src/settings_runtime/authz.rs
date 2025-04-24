// SPDX-FileCopyrightText: OpenTalk GmbH <mail@opentalk.eu>
//
// SPDX-License-Identifier: EUPL-1.2

use crate::settings_file;

#[derive(Debug, Clone, PartialEq, Eq)]
pub struct Authz {
    pub synchronize_controllers: bool,
}

impl Authz {
    pub(crate) fn from_settings_file(
        value: Option<settings_file::Authz>,
        is_rabbitmq_available: bool,
    ) -> Self {
        let synchronize_controllers = is_rabbitmq_available
            && value
                .and_then(|v| v.synchronize_controllers)
                .unwrap_or(true);
        Self {
            synchronize_controllers,
        }
    }
}
