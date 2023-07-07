// SPDX-FileCopyrightText: OpenTalk GmbH <mail@opentalk.eu>
//
// SPDX-License-Identifier: EUPL-1.2

use std::sync::Arc;

use anyhow::{Context, Result};
use aws_sdk_s3::primitives::ByteStream;
use bytes::Bytes;
use database::Db;
use db_storage::{
    assets::{Asset, NewAsset},
    rooms::Room,
};
use futures::Stream;
use types::core::{AssetId, RoomId};

use crate::ObjectStorage;

/// Save an asset in the long term storage
///
/// Creates a new database entry before after the asset in the configured S3 bucket.
pub async fn save_asset(
    storage: &ObjectStorage,
    db: Arc<Db>,
    room_id: RoomId,
    namespace: Option<&str>,
    filename: impl Into<String>,
    kind: impl Into<String>,
    data: impl Stream<Item = Result<Bytes>> + Unpin,
) -> Result<AssetId> {
    let namespace = namespace.map(Into::into);
    let filename = filename.into();
    let kind = kind.into();

    let asset_id = AssetId::generate();

    // upload to s3 storage
    storage
        .put(&asset_key(&asset_id), data)
        .await
        .context("failed to upload asset file to storage")?;

    // create db entry
    let mut db_conn = db.get_conn().await?;

    let room = Room::get(&mut db_conn, room_id).await?;

    let db_insert_res = NewAsset {
        id: asset_id,
        namespace,
        filename,
        kind,
        tenant_id: room.tenant_id,
    }
    .insert_for_room(&mut db_conn, room_id)
    .await;

    drop(db_conn);

    // rollback s3 storage if errors occurred
    if let Err(e) = db_insert_res {
        if let Err(err) = storage.delete(asset_key(&asset_id)).await {
            log::error!(
                "Failed to rollback s3 asset after database error, leaking asset: {}",
                &asset_key(&asset_id)
            );
            return Err(err);
        }

        return Err(e.into());
    }

    Ok(asset_id)
}

/// Get an asset from the object storage
pub async fn get_asset(storage: &ObjectStorage, asset_id: &AssetId) -> Result<ByteStream> {
    storage.get(asset_key(asset_id)).await
}

/// Delete an asset from the object storage
pub async fn delete_asset(
    storage: &ObjectStorage,
    db: Arc<Db>,
    room_id: RoomId,
    asset_id: AssetId,
) -> Result<()> {
    Asset::delete_by_id(&mut db.get_conn().await?, asset_id, room_id).await?;

    storage.delete(asset_key(&asset_id)).await
}

pub fn asset_key(asset_id: &AssetId) -> String {
    format!("assets/{asset_id}")
}
