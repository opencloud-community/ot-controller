// SPDX-FileCopyrightText: OpenTalk GmbH <mail@opentalk.eu>
//
// SPDX-License-Identifier: EUPL-1.2

use std::sync::Arc;

use async_trait::async_trait;
use opentalk_controller_settings::Settings;
use opentalk_database::{DatabaseError, Db, DbConnection};
use opentalk_db_storage::assets::{Asset, UpdateAsset};
use opentalk_log::{debug, info, warn};
use opentalk_signaling_core::{assets::asset_key, ExchangeHandle, ObjectStorage};
use serde::{Deserialize, Serialize};
use snafu::ResultExt;

use crate::{
    error::{ParameterLoadingSnafu, ParameterSerializingSnafu},
    Error, Job, JobParameters,
};

#[derive(Debug, PartialEq, Eq, Serialize, Deserialize)]
pub struct SyncStorageFilesParameters {
    #[serde(default)]
    missing_storage_file_handling: MissingStorageFileHandling,
}

#[derive(Debug, Clone, Copy, Default, PartialEq, Eq, Serialize, Deserialize)]
#[serde(rename_all = "snake_case")]
enum MissingStorageFileHandling {
    #[default]
    SetFileSizeToZero,
    DeleteAssetEntry,
}

impl JobParameters for SyncStorageFilesParameters {
    fn try_from_json(json: serde_json::Value) -> Result<Self, Error> {
        serde_json::from_value(json).context(ParameterLoadingSnafu)
    }

    fn to_json(&self) -> Result<serde_json::Value, Error> {
        serde_json::to_value(self).context(ParameterSerializingSnafu)
    }
}

/// A job that synchronizes file information from the S3 storage to our database
#[derive(Debug)]
pub struct SyncStorageFiles;

#[async_trait]
impl Job for SyncStorageFiles {
    type Parameters = SyncStorageFilesParameters;

    async fn execute(
        logger: &dyn log::Log,
        db: Arc<Db>,
        _exchange_handle: ExchangeHandle,
        settings: &Settings,
        parameters: Self::Parameters,
    ) -> Result<(), Error> {
        info!(log: logger, "Starting storage file synchronization job");
        debug!(log: logger, "Job parameters: {parameters:?}");

        let mut conn = db.get_conn().await?;

        let object_storage = ObjectStorage::new(&settings.minio).await?;

        sync_files(
            logger,
            &mut conn,
            &object_storage,
            &parameters.missing_storage_file_handling,
        )
        .await?;

        Ok(())
    }
}

async fn sync_files(
    logger: &dyn log::Log,
    conn: &mut DbConnection,
    object_storage: &ObjectStorage,
    missing_file_handling: &MissingStorageFileHandling,
) -> Result<(), Error> {
    let mut missing_asset_count = 0;
    let mut updated_asset_count = 0;

    let assets = Asset::get_all_ids_and_size(conn).await?;
    let total = assets.len();

    for (index, asset) in assets.iter().enumerate() {
        let asset_id = asset.0;
        let asset_size = asset.1;

        if index % (total / 5).max(1) == 0 && index != 0 {
            info!(log: logger,
                "Synchronized assets: {index}/{total} ...",
            );
        }

        let file_size = match object_storage
            .get_object_size_if_exists(asset_key(&asset_id))
            .await?
        {
            Some(file_size) => file_size,
            None => {
                missing_asset_count += 1;

                warn!(log: logger,"Missing asset {} in object storage", asset_id);
                match missing_file_handling {
                    MissingStorageFileHandling::SetFileSizeToZero => 0,
                    MissingStorageFileHandling::DeleteAssetEntry => {
                        warn!(log: logger,"Deleting asset {} from database", asset_id);
                        Asset::internal_delete_by_id(conn, &asset_id).await?;
                        continue;
                    }
                }
            }
        };

        if asset_size == file_size {
            continue;
        }

        let update = UpdateAsset {
            size: Some(file_size),
            filename: None,
        };

        if file_size == 0 {
            warn!(log: logger,"Setting file size of asset {} to zero", asset_id)
        }

        if let Err(e) = update.apply(conn, asset_id).await {
            if matches!(e, DatabaseError::NotFound) {
                warn!(log: logger,"Could not update asset {}, it appears to be deleted from the database", asset_id);
            } else {
                return Err(e.into());
            }
        }

        updated_asset_count += 1;
    }

    if total != 0 {
        info!(log: logger,"All {total} assets synchronized!");
    } else {
        info!(log: logger,"Nothing to synchronize, no assets found in database.");
    }

    if missing_asset_count == 0 {
        // finish without any further summary logs
        return Ok(());
    }

    match missing_file_handling {
        MissingStorageFileHandling::SetFileSizeToZero => {
            let changed_assets = updated_asset_count - missing_asset_count;

            if changed_assets > 0 {
                info!(log: logger,"Updated the file size of {changed_assets} changed asset(s)");
            }

            info!(log: logger,"Updated the file size to zero for {missing_asset_count} asset(s) that had no storage file")
        }
        MissingStorageFileHandling::DeleteAssetEntry => {
            if updated_asset_count > 0 {
                info!(log: logger,"Updated the file size of {updated_asset_count} changed asset(s)");
            }

            info!(log: logger,"Deleted {missing_asset_count} asset(s) that had no storage file")
        }
    }

    info!(log: logger,"Job finished!");

    Ok(())
}

#[cfg(test)]
mod tests {
    use bytes::Bytes;
    use futures::stream;
    use opentalk_controller_settings::MinIO;
    use opentalk_db_storage::assets::{Asset, NewAsset, UpdateAsset};
    use opentalk_signaling_core::{
        assets::{save_asset, NewAssetFileName},
        ChunkFormat, ObjectStorage, ObjectStorageError,
    };
    use opentalk_test_util::common::TestContext;
    use opentalk_types::core::{FileExtension, RoomId, Timestamp};
    use opentalk_types_common::assets::AssetId;

    use crate::jobs::sync_storage_files::{sync_files, MissingStorageFileHandling};

    /// The id for an asset that has no related storage file
    const LOST_ASSET_ID: AssetId = AssetId::from_u128(42);

    /// Test the database/storage file synchronization and delete assets that have to related storage object
    ///
    /// Requires a local MinIO instance
    ///
    /// `cargo test --package opentalk-jobs  -- --exact jobs::sync_storage_files::tests::sync_files_and_delete_missing --ignored`
    #[actix_rt::test]
    #[ignore]
    async fn sync_files_and_delete_missing() {
        sync_asset_test(499, &MissingStorageFileHandling::DeleteAssetEntry).await
    }

    /// Test the database/storage file synchronization and set the file size of assets that have no related storage object to zero
    ///
    /// Requires a local MinIO instance
    ///
    ///  `cargo test --package opentalk-jobs  -- --exact jobs::sync_storage_files::tests::sync_files_and_set_missing_to_zero --ignored`
    #[actix_rt::test]
    #[ignore]
    async fn sync_files_and_set_missing_to_zero() {
        sync_asset_test(99, &MissingStorageFileHandling::SetFileSizeToZero).await
    }

    /// Test the database/storage file synchronization and with a low amount of assets
    ///
    /// `cargo test --package opentalk-jobs  -- --exact jobs::sync_storage_files::tests::sync_low_asset_count --ignored`
    #[actix_rt::test]
    #[ignore]
    async fn sync_low_asset_count() {
        sync_asset_test(2, &MissingStorageFileHandling::DeleteAssetEntry).await
    }

    /// Test the database/storage file synchronization with no assets in the database
    ///
    /// `cargo test --package opentalk-jobs  -- --exact jobs::sync_storage_files::tests::sync_zero_assets --ignored`
    #[actix_rt::test]
    #[ignore]
    async fn sync_zero_assets() {
        let test_ctx = TestContext::default().await;

        let minio = MinIO {
            uri: "http://localhost:9555".into(),
            bucket: "controller".into(),
            access_key: "minioadmin".into(),
            secret_key: "minioadmin".into(),
        };

        let object_storage = ObjectStorage::new(&minio).await.unwrap();

        let mut conn = test_ctx.db_ctx.db.get_conn().await.unwrap();

        sync_files(
            log::logger(),
            &mut conn,
            &object_storage,
            &MissingStorageFileHandling::DeleteAssetEntry,
        )
        .await
        .unwrap();
    }

    async fn sync_asset_test(
        valid_asset_count: usize,
        missing_file_handling: &MissingStorageFileHandling,
    ) {
        let test_ctx = TestContext::default().await;

        let minio = MinIO {
            uri: "http://localhost:9555".into(),
            bucket: "controller".into(),
            access_key: "minioadmin".into(),
            secret_key: "minioadmin".into(),
        };

        let object_storage = ObjectStorage::new(&minio).await.unwrap();

        prepare_db_and_storage(&test_ctx, &object_storage, valid_asset_count).await;

        let mut conn = test_ctx.db_ctx.db.get_conn().await.unwrap();

        sync_files(
            log::logger(),
            &mut conn,
            &object_storage,
            missing_file_handling,
        )
        .await
        .unwrap();

        let assets = Asset::get_all_ids_and_size(&mut conn).await.unwrap();

        assert_eq!(assets.len(), valid_asset_count);

        // ensure that the asset without the storage file was deleted
        assert!(!assets.iter().any(|asset| asset.0 == LOST_ASSET_ID))
    }

    /// Creates multiple valid database assets with their associated storage object as well as one invalid database asset.
    async fn prepare_db_and_storage(
        test_ctx: &TestContext,
        object_storage: &ObjectStorage,
        asset_count: usize,
    ) {
        let db_ctx = &test_ctx.db_ctx;

        let mut conn = db_ctx.db.get_conn().await.unwrap();

        let user = db_ctx.create_test_user(0, Vec::new()).await.unwrap();

        let room = db_ctx
            .create_test_room(RoomId::nil(), user.id, false)
            .await
            .unwrap();

        for i in 0..asset_count {
            let data = stream::iter(vec![Ok::<Bytes, ObjectStorageError>(Bytes::from_static(
                b"data",
            ))]);

            let kind = "test".parse().unwrap();
            let filename = NewAssetFileName::new(kind, Timestamp::now(), FileExtension::pdf());

            let (asset_id, _filename) = save_asset(
                object_storage,
                db_ctx.db.clone(),
                room.id,
                None,
                filename,
                data,
                ChunkFormat::Data,
            )
            .await
            .unwrap();

            // The first and every 40th asset will have a wrong file size
            if i % 40 == 0 {
                UpdateAsset {
                    size: Some(23456),
                    filename: None,
                }
                .apply(&mut conn, asset_id)
                .await
                .unwrap();
            }
        }

        // create an asset that does not have a related file in the object storage
        let asset = NewAsset {
            id: LOST_ASSET_ID,
            namespace: None,
            kind: "LostAsset".into(),
            filename: "does_not_exist_in_storage.txt".into(),
            tenant_id: user.tenant_id,
            size: 42,
        };

        asset.insert_for_room(&mut conn, room.id).await.unwrap();
    }
}
