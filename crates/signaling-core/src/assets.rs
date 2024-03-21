// SPDX-FileCopyrightText: OpenTalk GmbH <mail@opentalk.eu>
//
// SPDX-License-Identifier: EUPL-1.2

use std::{
    pin::Pin,
    sync::Arc,
    task::{self, Poll},
};

use aws_sdk_s3::primitives::{ByteStream, ByteStreamError};
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
use opentalk_types::{
    api::error::ApiError,
    core::{AssetId, RoomId, UserId},
};
use snafu::{IntoError, ResultExt, Snafu};

use crate::{ObjectStorage, ObjectStorageError};

#[derive(Debug, Snafu)]
pub enum AssetError {
    #[snafu(display("Database connection failed: {source}"))]
    DbConnection {
        source: opentalk_database::DatabaseError,
    },

    #[snafu(display("Database query failed: {source}"))]
    DbQuery {
        source: opentalk_database::DatabaseError,
    },

    #[snafu(display("Failed to upload asset to storage: {source}"))]
    ObjectStorage {
        source: crate::object_storage::ObjectStorageError,
    },

    #[snafu(display("File size too big"))]
    FileSize { source: std::num::TryFromIntError },

    #[snafu(display("The storage quota was exceeded"))]
    AssetStorageExceeded,

    #[snafu(display("The storage quota was exceeded"))]
    // Use AssetError instead of Self, since self will refer to the RollbackSnafu inside the expanded code
    Rollback {
        /// The error that caused the rollback to fail
        #[snafu(source(from(AssetError, Box::new)))]
        source: Box<AssetError>,
        /// The error that required a rollback
        rollback_reason: Box<AssetError>,
    },
}

impl From<AssetError> for ApiError {
    fn from(value: AssetError) -> Self {
        log::error!("REST API threw internal error: {value:?}");
        ApiError::internal()
    }
}

type Result<T, E = AssetError> = std::result::Result<T, E>;

/// Save an asset in the long term storage
///
/// Creates a new database entry before after the asset in the configured S3 bucket.
pub async fn save_asset<E>(
    storage: &ObjectStorage,
    db: Arc<Db>,
    room_id: RoomId,
    namespace: Option<&str>,
    filename: impl Into<String>,
    kind: impl Into<String>,
    data: impl Stream<Item = Result<Bytes, E>> + Unpin,
) -> Result<AssetId>
where
    ObjectStorageError: From<E>,
{
    let mut conn = db.get_conn().await.context(DbConnectionSnafu)?;
    let namespace = namespace.map(Into::into);
    let filename = filename.into();
    let kind = kind.into();

    let room = prepare_storage(room_id, &mut conn).await?;

    let asset_id = AssetId::generate();

    // Upload to s3 storage
    let size: Result<i64, _> = storage
        .put(&asset_key(&asset_id), data)
        .await
        .context(ObjectStorageSnafu)?
        .try_into()
        .context(FileSizeSnafu);

    let size = match size {
        Ok(size) => size,
        Err(e) => {
            drop(conn);
            rollback_object_storage(storage, &asset_id).await?;
            return Err(e);
        }
    };

    // Create a database entry for the uploaded asset
    let result =
        insert_asset_into_database(&mut conn, namespace, filename, kind, asset_id, room, size)
            .await
            .context(DbQuerySnafu);

    if let Err(e) = result {
        drop(conn);
        // if there was an error, we roll back and return the original error.
        // if the rollback fails, we return a rollback error with the cause of
        // the rollback and the reason why the rollback failed.
        return match rollback_object_storage(storage, &asset_id).await {
            Ok(_) => Err(e),
            Err(rollback_err) => Err(rollback_err)
                .with_context(|_| RollbackSnafu::<AssetError> { rollback_reason: e }),
        };
    }

    Ok(asset_id)
}

async fn rollback_object_storage(storage: &ObjectStorage, asset_id: &AssetId) -> Result<()> {
    log::info!("Rollback asset upload since room update failed");
    if let Err(rollback_err) = storage.delete(asset_key(asset_id)).await {
        log::error!(
            "Failed to rollback s3 asset after database error, leaking asset: {}",
            &asset_key(asset_id)
        );
        Err(ObjectStorageSnafu.into_error(rollback_err))
    } else {
        Ok(())
    }
}

async fn insert_asset_into_database(
    db_conn: &mut DbConnection,
    namespace: Option<String>,
    filename: String,
    kind: String,
    asset_id: AssetId,
    room: Room,
    size: i64,
) -> opentalk_database::Result<Asset> {
    NewAsset {
        id: asset_id,
        namespace,
        filename,
        kind,
        tenant_id: room.tenant_id,
        size,
    }
    .insert_for_room(db_conn, room.id)
    .await
}

async fn prepare_storage(room_id: RoomId, conn: &mut DbConnection) -> Result<Room, AssetError> {
    let room = Room::get(conn, room_id).await.context(DbQuerySnafu)?;
    verify_storage_usage(conn, room.created_by).await?;
    Ok(room)
}

pub struct ByStreamExt(ByteStream);

impl futures::stream::Stream for ByStreamExt {
    type Item = Result<Bytes, ByteStreamError>;

    fn poll_next(mut self: Pin<&mut Self>, cx: &mut task::Context<'_>) -> Poll<Option<Self::Item>> {
        Pin::new(&mut self.0).poll_next(cx)
    }
}

/// Get an asset from the object storage
pub async fn get_asset(
    storage: &ObjectStorage,
    asset_id: &AssetId,
) -> Result<ByStreamExt, crate::object_storage::ObjectStorageError> {
    let stream = storage.get(asset_key(asset_id)).await?;
    Ok(ByStreamExt(stream))
}

/// Delete an asset from the object storage
pub async fn delete_asset(
    storage: &ObjectStorage,
    db: Arc<Db>,
    room_id: RoomId,
    asset_id: AssetId,
) -> Result<()> {
    let mut conn = db.get_conn().await.context(DbConnectionSnafu)?;
    Asset::delete_by_id(&mut conn, asset_id, room_id)
        .await
        .context(DbQuerySnafu)?;

    storage
        .delete(asset_key(&asset_id))
        .await
        .context(ObjectStorageSnafu)
}

pub fn asset_key(asset_id: &AssetId) -> String {
    format!("assets/{asset_id}")
}

/// Verify that the storage quota wasn't exhausted. Files don't need to fit into the remaining quota,
/// there only needs to be available quota.
pub async fn verify_storage_usage(db_conn: &mut DbConnection, user_id: UserId) -> Result<()> {
    let used_storage = User::get_used_storage(db_conn, &user_id)
        .await
        .context(DbQuerySnafu)?;
    let user_tariff = Tariff::get_by_user_id(db_conn, &user_id)
        .await
        .context(DbQuerySnafu)?;

    if let Some(max_storage) = user_tariff.quotas.0.get("max_storage") {
        if used_storage > BigDecimal::from(*max_storage) {
            return AssetStorageExceededSnafu.fail();
        }
    }

    Ok(())
}
