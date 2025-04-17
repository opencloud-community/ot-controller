// SPDX-FileCopyrightText: OpenTalk GmbH <mail@opentalk.eu>
//
// SPDX-License-Identifier: EUPL-1.2

use serde::Deserialize;

use super::TenantAssignment;

#[derive(Default, Debug, Clone, PartialEq, Eq, Deserialize)]
pub struct Tenants {
    #[serde(default, flatten)]
    pub assignment: TenantAssignment,
}
