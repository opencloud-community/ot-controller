// SPDX-FileCopyrightText: OpenTalk GmbH <mail@opentalk.eu>
//
// SPDX-License-Identifier: EUPL-1.2

use opentalk_db_storage::assets::{Asset, NewAsset, UpdateAsset};
use opentalk_types_common::{
    assets::{AssetId, AssetSorting},
    events::EventId,
    order::Ordering,
    rooms::RoomId,
    users::UserId,
};

use crate::Result;

/// A trait for retrieving and storing event entities.
#[async_trait::async_trait]
pub trait AssetInventory {
    /// Create an asset for a room.
    async fn create_asset_for_room(&mut self, room_id: RoomId, asset: NewAsset) -> Result<Asset>;

    /// Delete an asset from a room.
    async fn delete_asset_from_room(&mut self, room_id: RoomId, asset_id: AssetId) -> Result<()>;

    /// Get an asset for a room.
    async fn get_asset_for_room(&mut self, room_id: RoomId, asset_id: AssetId) -> Result<Asset>;

    /// Get all asset ids and their size
    async fn get_all_assets_with_size(&mut self) -> Result<Vec<(AssetId, i64)>>;

    /// Get all assets for a room, paginated.
    ///
    /// Returns a tuple of the loaded assets, and the overall count of available assets.
    async fn get_all_assets_for_room_paginated(
        &mut self,
        room: RoomId,
        per_page: i64,
        page: i64,
    ) -> Result<(Vec<Asset>, i64)>;

    /// Get all asset ids for a room.
    async fn get_all_asset_ids_for_room(&mut self, room_id: RoomId) -> Result<Vec<AssetId>>;

    /// Get all assets associated with rooms owned by a specific user, paginated and ordered.
    ///
    /// Returns a tuple of the loaded assets, and the overall count of available assets.
    async fn get_all_assets_for_room_owner_paginated_ordered(
        &mut self,
        user_id: UserId,
        limit: i64,
        page: i64,
        sort: AssetSorting,
        order: Ordering,
    ) -> Result<(Vec<(Asset, RoomId, Option<EventId>)>, i64)>;

    /// Update an asset.
    ///
    /// Will return [`Ok(None)`] if no asset with the given `asset_id` is found.
    async fn update_asset(
        &mut self,
        asset_id: AssetId,
        asset: UpdateAsset,
    ) -> Result<Option<Asset>>;

    /// Internal deletion of assets.
    async fn delete_asset_by_id_internal(&mut self, asset_id: AssetId) -> Result<()>;

    /// Internal deletion of assets.
    async fn delete_assets_by_ids(&mut self, asset_ids: &[AssetId]) -> Result<()>;
}
