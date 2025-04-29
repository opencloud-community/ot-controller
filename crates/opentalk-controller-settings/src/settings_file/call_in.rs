// SPDX-FileCopyrightText: OpenTalk GmbH <mail@opentalk.eu>
//
// SPDX-License-Identifier: EUPL-1.2

use serde::Deserialize;

#[derive(Clone, Debug, PartialEq, Eq, Deserialize)]
pub(crate) struct CallIn {
    pub tel: String,
    pub enable_phone_mapping: bool,
    pub default_country_code: phonenumber::country::Id,
}
