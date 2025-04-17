// SPDX-FileCopyrightText: OpenTalk GmbH <mail@opentalk.eu>
//
// SPDX-License-Identifier: EUPL-1.2

use rustc_hash::FxHashSet;
use serde::Deserialize;

#[derive(Default, Debug, Clone, PartialEq, Eq, Deserialize)]
pub struct TariffStatusMapping {
    pub downgraded_tariff_name: String,
    pub default: FxHashSet<String>,
    pub paid: FxHashSet<String>,
    pub downgraded: FxHashSet<String>,
}
