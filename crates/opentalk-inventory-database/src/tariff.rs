// SPDX-FileCopyrightText: OpenTalk GmbH <mail@opentalk.eu>
//
// SPDX-License-Identifier: EUPL-1.2

use opentalk_database::OptionalExt as _;
use opentalk_db_storage::tariffs::{ExternalTariff, ExternalTariffId, Tariff};
use opentalk_inventory::{TariffInventory, error::StorageBackendSnafu};
use opentalk_types_common::{tariffs::TariffId, users::UserId};
use snafu::ResultExt as _;

use crate::{DatabaseConnection, Result};

#[async_trait::async_trait]
impl TariffInventory for DatabaseConnection {
    #[tracing::instrument(err, skip_all)]
    async fn get_tariff(&mut self, tariff_id: TariffId) -> Result<Tariff> {
        Tariff::get(&mut self.inner, tariff_id)
            .await
            .context(StorageBackendSnafu)
    }

    #[tracing::instrument(err, skip_all)]
    async fn get_tariff_by_name(&mut self, tariff_name: &str) -> Result<Tariff> {
        Tariff::get_by_name(&mut self.inner, tariff_name)
            .await
            .context(StorageBackendSnafu)
    }

    #[tracing::instrument(err, skip_all)]
    async fn get_tariff_for_user(&mut self, user_id: UserId) -> Result<Tariff> {
        Tariff::get_by_user_id(&mut self.inner, &user_id)
            .await
            .context(StorageBackendSnafu)
    }

    #[tracing::instrument(err, skip_all)]
    async fn get_tariff_by_external_tariff_id(
        &mut self,
        external_tariff_id: &ExternalTariffId,
    ) -> Result<Option<Tariff>> {
        Tariff::get_by_external_id(&mut self.inner, external_tariff_id)
            .await
            .optional()
            .context(StorageBackendSnafu)
    }

    #[tracing::instrument(err, skip_all)]
    async fn get_all_external_tariff_ids_for_tariff(
        &mut self,
        tariff_id: TariffId,
    ) -> Result<Vec<ExternalTariffId>> {
        ExternalTariff::get_all_for_tariff(&mut self.inner, tariff_id)
            .await
            .context(StorageBackendSnafu)
    }
}
