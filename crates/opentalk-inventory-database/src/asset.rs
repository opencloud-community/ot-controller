// SPDX-FileCopyrightText: OpenTalk GmbH <mail@opentalk.eu>
//
// SPDX-License-Identifier: EUPL-1.2

use opentalk_database::DatabaseError;
use opentalk_db_storage::assets::{self, Asset, NewAsset, UpdateAsset};
use opentalk_inventory::{AssetInventory, error::StorageBackendSnafu};
use opentalk_types_common::{
    assets::{AssetId, AssetSorting},
    events::EventId,
    order::Ordering,
    rooms::RoomId,
    users::UserId,
};
use snafu::ResultExt as _;

use crate::{DatabaseConnection, Result};

#[async_trait::async_trait]
impl AssetInventory for DatabaseConnection {
    #[tracing::instrument(err, skip_all)]
    async fn create_asset_for_room(&mut self, room_id: RoomId, asset: NewAsset) -> Result<Asset> {
        asset
            .insert_for_room(&mut self.inner, room_id)
            .await
            .context(StorageBackendSnafu)
    }

    #[tracing::instrument(err, skip_all)]
    async fn delete_asset_from_room(&mut self, room_id: RoomId, asset_id: AssetId) -> Result<()> {
        Asset::delete_by_id(&mut self.inner, asset_id, room_id)
            .await
            .context(StorageBackendSnafu)
    }

    #[tracing::instrument(err, skip_all)]
    async fn get_asset_for_room(&mut self, room_id: RoomId, asset_id: AssetId) -> Result<Asset> {
        Asset::get(&mut self.inner, asset_id, room_id)
            .await
            .context(StorageBackendSnafu)
    }

    #[tracing::instrument(err, skip_all)]
    async fn get_all_assets_with_size(&mut self) -> Result<Vec<(AssetId, i64)>> {
        Asset::get_all_ids_and_size(&mut self.inner)
            .await
            .context(StorageBackendSnafu)
    }

    #[tracing::instrument(err, skip_all)]
    async fn get_all_assets_for_room_paginated(
        &mut self,
        room_id: RoomId,
        per_page: i64,
        page: i64,
    ) -> Result<(Vec<Asset>, i64)> {
        Asset::get_all_for_room_paginated(&mut self.inner, room_id, per_page, page)
            .await
            .context(StorageBackendSnafu)
    }

    #[tracing::instrument(err, skip_all)]
    async fn get_all_asset_ids_for_room(&mut self, room_id: RoomId) -> Result<Vec<AssetId>> {
        Asset::get_all_ids_for_room(&mut self.inner, room_id)
            .await
            .context(StorageBackendSnafu)
    }

    #[tracing::instrument(err, skip_all)]
    async fn get_all_assets_for_room_owner_paginated_ordered(
        &mut self,
        user_id: UserId,
        limit: i64,
        page: i64,
        sort: AssetSorting,
        order: Ordering,
    ) -> Result<(Vec<(Asset, RoomId, Option<EventId>)>, i64)> {
        assets::get_all_for_room_owner_paginated_ordered(
            &mut self.inner,
            user_id,
            limit,
            page,
            sort,
            order,
        )
        .await
        .context(StorageBackendSnafu)
    }

    #[tracing::instrument(err, skip_all)]
    async fn update_asset(
        &mut self,
        asset_id: AssetId,
        asset: UpdateAsset,
    ) -> Result<Option<Asset>> {
        match asset.apply(&mut self.inner, asset_id).await {
            Ok(asset) => Ok(Some(asset)),
            Err(DatabaseError::NotFound) => Ok(None),
            Err(e) => Err(e).context(StorageBackendSnafu),
        }
    }

    #[tracing::instrument(err, skip_all)]
    async fn delete_asset_by_id_internal(&mut self, asset_id: AssetId) -> Result<()> {
        Asset::internal_delete_by_id(&mut self.inner, &asset_id)
            .await
            .context(StorageBackendSnafu)
    }

    #[tracing::instrument(err, skip_all)]
    async fn delete_assets_by_ids(&mut self, asset_ids: &[AssetId]) -> Result<()> {
        Asset::delete_by_ids(&mut self.inner, asset_ids)
            .await
            .context(StorageBackendSnafu)
    }
}
