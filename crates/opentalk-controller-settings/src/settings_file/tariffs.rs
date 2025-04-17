// SPDX-FileCopyrightText: OpenTalk GmbH <mail@opentalk.eu>
//
// SPDX-License-Identifier: EUPL-1.2

use serde::Deserialize;

use super::{TariffAssignment, TariffStatusMapping};

#[derive(Default, Debug, Clone, PartialEq, Eq, Deserialize)]
pub struct Tariffs {
    #[serde(default, flatten)]
    pub assignment: TariffAssignment,

    #[serde(default)]
    pub status_mapping: Option<TariffStatusMapping>,
}
