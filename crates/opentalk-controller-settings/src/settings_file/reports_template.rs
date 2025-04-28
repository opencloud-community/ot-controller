// SPDX-FileCopyrightText: OpenTalk GmbH <mail@opentalk.eu>
//
// SPDX-License-Identifier: EUPL-1.2

use serde::Deserialize;

#[derive(Clone, Debug, Deserialize, Default, PartialEq, Eq)]
#[serde(rename_all = "snake_case")]
pub(crate) enum ReportsTemplate {
    /// Use the Template included with the application.
    #[default]
    BuiltIn,

    /// Use the Template provided by the user configuration.
    Inline(String),
}
