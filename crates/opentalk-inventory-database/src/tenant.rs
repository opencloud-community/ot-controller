// SPDX-FileCopyrightText: OpenTalk GmbH <mail@opentalk.eu>
//
// SPDX-License-Identifier: EUPL-1.2

use opentalk_db_storage::tenants::{get_or_create_tenant_by_oidc_id, OidcTenantId, Tenant};
use opentalk_inventory::{error::StorageBackendSnafu, TenantInventory};
use opentalk_types_common::tenants::TenantId;
use snafu::ResultExt as _;

use crate::{DatabaseConnection, Result};

#[async_trait::async_trait]
impl TenantInventory for DatabaseConnection {
    #[tracing::instrument(err, skip_all)]
    async fn get_tenant(&mut self, tenant_id: TenantId) -> Result<Tenant> {
        Tenant::get(&mut self.inner, tenant_id)
            .await
            .context(StorageBackendSnafu)
    }

    #[tracing::instrument(err, skip_all)]
    async fn get_or_create_tenant_by_oidc_id(
        &mut self,
        oidc_tenant_id: &OidcTenantId,
    ) -> Result<Tenant> {
        get_or_create_tenant_by_oidc_id(&mut self.inner, oidc_tenant_id)
            .await
            .context(StorageBackendSnafu)
    }
}
