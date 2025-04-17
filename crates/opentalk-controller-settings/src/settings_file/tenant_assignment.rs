// SPDX-FileCopyrightText: OpenTalk GmbH <mail@opentalk.eu>
//
// SPDX-License-Identifier: EUPL-1.2

use serde::Deserialize;

#[derive(Debug, Clone, PartialEq, Eq, Deserialize)]
#[serde(rename_all = "snake_case", tag = "assignment")]
pub enum TenantAssignment {
    Static {
        static_tenant_id: String,
    },
    ByExternalTenantId {
        #[serde(default = "default_external_tenant_id_user_attribute_name")]
        external_tenant_id_user_attribute_name: String,
    },
}

fn default_external_tenant_id_user_attribute_name() -> String {
    "tenant_id".to_owned()
}

impl Default for TenantAssignment {
    fn default() -> Self {
        Self::Static {
            static_tenant_id: String::from("OpenTalkDefaultTenant"),
        }
    }
}
