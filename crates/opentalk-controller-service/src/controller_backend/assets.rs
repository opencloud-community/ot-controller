// SPDX-FileCopyrightText: OpenTalk GmbH <mail@opentalk.eu>
//
// SPDX-License-Identifier: EUPL-1.2

use bytes::Bytes;
use futures_core::Stream;
use opentalk_controller_utils::CaptureApiError;
use opentalk_db_storage::assets::Asset;
use opentalk_signaling_core::{
    assets::{delete_asset, get_asset, save_asset, ByStreamExt, NewAssetFileName},
    ChunkFormat, ObjectStorageError,
};
use opentalk_types_api_v1::{
    assets::AssetResource, pagination::PagePaginationQuery,
    rooms::by_room_id::assets::RoomsByRoomIdAssetsGetResponseBody,
};
use opentalk_types_common::{assets::AssetId, modules::ModuleId, rooms::RoomId};

use crate::{helpers::asset_to_asset_resource, ControllerBackend};

impl ControllerBackend {
    pub(super) async fn get_room_assets(
        &self,
        room_id: RoomId,
        pagination: &PagePaginationQuery,
    ) -> Result<(RoomsByRoomIdAssetsGetResponseBody, i64), CaptureApiError> {
        let mut conn = self.db.get_conn().await?;

        let (assets, asset_count) = Asset::get_all_for_room_paginated(
            &mut conn,
            room_id,
            pagination.per_page,
            pagination.page,
        )
        .await?;

        let assets = assets.into_iter().map(asset_to_asset_resource).collect();
        let assets = RoomsByRoomIdAssetsGetResponseBody(assets);

        Ok((assets, asset_count))
    }

    pub(super) async fn get_room_asset(
        &self,
        room_id: RoomId,
        asset_id: AssetId,
    ) -> Result<ByStreamExt, CaptureApiError> {
        let mut conn = self.db.get_conn().await?;

        let asset = Asset::get(&mut conn, asset_id, room_id).await?;

        let stream = get_asset(&self.storage, &asset.id).await?;

        Ok(stream)
    }

    pub(super) async fn create_room_asset(
        &self,
        room_id: RoomId,
        filename: NewAssetFileName,
        namespace: Option<ModuleId>,
        data: Box<dyn Stream<Item = Result<Bytes, ObjectStorageError>> + Unpin>,
    ) -> Result<AssetResource, CaptureApiError> {
        let (asset_id, _filename) = save_asset(
            &self.storage.clone(),
            self.db.clone(),
            room_id,
            namespace,
            filename,
            data,
            ChunkFormat::Data,
        )
        .await?;

        let asset = Asset::get(&mut self.db.get_conn().await?, asset_id, room_id).await?;

        Ok(asset_to_asset_resource(asset))
    }

    pub(super) async fn delete_room_asset(
        &self,
        room_id: RoomId,
        asset_id: AssetId,
    ) -> Result<(), CaptureApiError> {
        delete_asset(&self.storage, &self.db, room_id, asset_id).await?;

        Ok(())
    }
}
