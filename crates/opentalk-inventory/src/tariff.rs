// SPDX-FileCopyrightText: OpenTalk GmbH <mail@opentalk.eu>
//
// SPDX-License-Identifier: EUPL-1.2

use opentalk_db_storage::tariffs::{ExternalTariffId, Tariff};
use opentalk_types_common::{tariffs::TariffId, users::UserId};

use crate::Result;

/// A trait for retrieving and storing tariff entities.
#[async_trait::async_trait]
pub trait TariffInventory {
    /// Get a tariff by its id.
    async fn get_tariff(&mut self, tariff_id: TariffId) -> Result<Tariff>;

    /// Get a tariff by its name.
    async fn get_tariff_by_name(&mut self, name: &str) -> Result<Tariff>;

    /// Get the tariff of a user.
    async fn get_tariff_for_user(&mut self, user_id: UserId) -> Result<Tariff>;

    /// Get a tariff by its external tariff id.
    async fn get_tariff_by_external_tariff_id(
        &mut self,
        external_tariff_id: &ExternalTariffId,
    ) -> Result<Option<Tariff>>;

    /// Get all external tariff ids for a tariff.
    async fn get_all_external_tariff_ids_for_tariff(
        &mut self,
        tariff_id: TariffId,
    ) -> Result<Vec<ExternalTariffId>>;
}
