// SPDX-FileCopyrightText: OpenTalk GmbH <mail@opentalk.eu>
//
// SPDX-License-Identifier: EUPL-1.2

use std::{
    mem,
    pin::Pin,
    sync::Arc,
    task::{self, Poll},
};

use anyhow::{anyhow, Context, Result};
use aws_sdk_s3::primitives::ByteStream;
use bigdecimal::BigDecimal;
use bytes::Bytes;
use futures::Stream;
use opentalk_database::{Db, DbConnection};
use opentalk_db_storage::{
    assets::{Asset, NewAsset},
    rooms::Room,
    tariffs::Tariff,
    users::User,
};
use opentalk_types::core::{AssetId, RoomId, UserId};
use thiserror::Error;

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
    let mut db_conn = db.get_conn().await.context("failed to get db connection")?;
    let namespace = namespace.map(Into::into);
    let filename = filename.into();
    let kind = kind.into();

    let asset_id = AssetId::generate();
    let size = mem::size_of_val(&data) as i64;

    let room = Room::get(&mut db_conn, room_id).await?;
    verify_storage_usage(&mut db_conn, room.created_by).await?;

    // upload to s3 storage
    storage
        .put(&asset_key(&asset_id), data)
        .await
        .context("failed to upload asset file to storage")?;

    // create db entry

    let room = Room::get(&mut db_conn, room_id).await?;

    let db_insert_res = NewAsset {
        id: asset_id,
        namespace,
        filename,
        kind,
        tenant_id: room.tenant_id,
        size,
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

pub struct ByStreamExt(ByteStream);

impl futures::stream::Stream for ByStreamExt {
    type Item = Result<Bytes, anyhow::Error>;

    fn poll_next(mut self: Pin<&mut Self>, cx: &mut task::Context<'_>) -> Poll<Option<Self::Item>> {
        Pin::new(&mut self.0).poll_next(cx).map_err(|e| anyhow!(e))
    }
}

/// Get an asset from the object storage
pub async fn get_asset(storage: &ObjectStorage, asset_id: &AssetId) -> Result<ByStreamExt> {
    Ok(ByStreamExt(storage.get(asset_key(asset_id)).await?))
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

pub async fn verify_storage_usage(db_conn: &mut DbConnection, user_id: UserId) -> Result<()> {
    let used_storage = User::get_used_storage(db_conn, &user_id).await?;
    let user_tariff = Tariff::get_by_user_id(db_conn, &user_id).await?;

    if let Some(max_storage) = user_tariff.quotas.0.get("max_storage") {
        if used_storage > BigDecimal::from(*max_storage) {
            return Err(AssetError::StorageExceeded.into());
        }
    }

    Ok(())
}

/// Error when attempting to access an asset
#[derive(Error, Debug, Clone, Copy)]
pub enum AssetError {
    /// The user has exceeded their storage
    #[error("The storage limit of your tariff is exceeded.")]
    StorageExceeded,
}
