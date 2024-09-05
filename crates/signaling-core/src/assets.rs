// SPDX-FileCopyrightText: OpenTalk GmbH <mail@opentalk.eu>
//
// SPDX-License-Identifier: EUPL-1.2

use std::{
    fmt::Display,
    pin::Pin,
    str::FromStr,
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
use opentalk_types::api::error::ApiError;
use opentalk_types_common::{
    assets::{AssetId, FileExtension},
    rooms::RoomId,
    tariffs::QuotaType,
    time::Timestamp,
    users::UserId,
};
use snafu::{IntoError, ResultExt, Snafu};

use crate::{object_storage::ChunkFormat, ObjectStorage, ObjectStorageError};

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

pub const MIN_ASSET_FILE_KIND_LENGTH: usize = 1;
pub const MAX_ASSET_FILE_KIND_LENGTH: usize = 20;

#[derive(Debug, Clone, Default, PartialEq, Eq, derive_more::Display)]
pub struct AssetFileKind(String);

#[derive(Debug, Snafu)]
pub enum ParseAssetFileKindError {
    #[snafu(display("AssetFileKind must be at least {min_length} characters long"))]
    TooShort { min_length: usize },

    #[snafu(display("AssetFileKind must not be longer than {max_length} characters"))]
    TooLong { max_length: usize },

    #[snafu(display("AssetFileKind only allows alphanumeric ascii characters or '_'"))]
    InvalidCharacters,
}

impl FromStr for AssetFileKind {
    type Err = ParseAssetFileKindError;

    fn from_str(s: &str) -> Result<Self, Self::Err> {
        if s.len() < MIN_ASSET_FILE_KIND_LENGTH {
            return Err(ParseAssetFileKindError::TooShort {
                min_length: MIN_ASSET_FILE_KIND_LENGTH,
            });
        }
        if s.len() > MAX_ASSET_FILE_KIND_LENGTH {
            return Err(ParseAssetFileKindError::TooLong {
                max_length: MAX_ASSET_FILE_KIND_LENGTH,
            });
        }
        if s.chars().any(|c| !(c.is_ascii_alphanumeric() || c == '_')) {
            return Err(ParseAssetFileKindError::InvalidCharacters);
        }
        Ok(AssetFileKind(s.into()))
    }
}

const MAX_ASSET_FILE_NAME_LENGTH: usize = 100;

pub struct NewAssetFileName {
    event_title: Option<String>,
    kind: AssetFileKind,
    timestamp: Timestamp,
    extension: FileExtension,
}

impl NewAssetFileName {
    pub fn new(kind: AssetFileKind, timestamp: Timestamp, extension: FileExtension) -> Self {
        Self {
            event_title: None,
            kind,
            timestamp,
            extension,
        }
    }

    pub fn new_with_event_title(
        event_title: Option<String>,
        kind: AssetFileKind,
        timestamp: Timestamp,
        extension: FileExtension,
    ) -> Self {
        Self {
            event_title,
            kind,
            timestamp,
            extension,
        }
    }

    fn sanitize_event_title_for_filename(s: &str, max_length: usize) -> String {
        let end = std::cmp::min(max_length, s.len());
        s[..end].replace(
            |c: char| !(c.is_alphanumeric() || ['.', '_', '-', ' '].contains(&c)),
            "_",
        )
    }
}

impl Display for NewAssetFileName {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        let file_name_fixed_part = format!(
            "{}_{}{}",
            self.kind,
            self.timestamp.to_string_for_filename(),
            self.extension.to_string_with_leading_dot()
        );
        match &self.event_title {
            Some(event_title) if !event_title.is_empty() => {
                let max_length =
                    MAX_ASSET_FILE_NAME_LENGTH.saturating_sub(file_name_fixed_part.len() + 1);
                write!(
                    f,
                    "{}_{}",
                    Self::sanitize_event_title_for_filename(event_title, max_length),
                    file_name_fixed_part
                )
            }
            _ => {
                write!(f, "{file_name_fixed_part}")
            }
        }
    }
}

/// Save an asset in the long term storage
///
/// Creates a new database entry before after the asset in the configured S3 bucket.
///
/// If the filename passed in does not have an event title set, this function
/// will attempt to load the title from the event associated with the room if
/// there is any. If no event is associated with the room, the event title will
/// stay empty.
///
/// Returns a tuple containing the asset id and the filename on success.
pub async fn save_asset<E>(
    storage: &ObjectStorage,
    db: Arc<Db>,
    room_id: RoomId,
    namespace: Option<&str>,
    mut filename: NewAssetFileName,
    data: impl Stream<Item = Result<Bytes, E>> + Unpin,
    chunk_format: ChunkFormat,
) -> Result<(AssetId, String)>
where
    ObjectStorageError: From<E>,
{
    let mut conn = db.get_conn().await.context(DbConnectionSnafu)?;
    let namespace = namespace.map(Into::into);

    let room = prepare_storage(room_id, &mut conn).await?;

    let asset_id = AssetId::generate();

    // Upload to s3 storage
    let size: Result<i64, _> = storage
        .put(&asset_key(&asset_id), data, chunk_format)
        .await
        .context(ObjectStorageSnafu)
        .and_then(|size| size.try_into().context(FileSizeSnafu));

    let size = match size {
        Ok(size) => size,
        Err(e) => {
            drop(conn);
            rollback_object_storage(storage, &asset_id).await?;
            return Err(e);
        }
    };

    if filename.event_title.is_none() {
        filename.event_title = opentalk_db_storage::events::Event::get_for_room(&mut conn, room.id)
            .await
            .context(DbQuerySnafu)?
            .map(|e| e.title);
    }

    let kind = filename.kind.clone();
    let filename = filename.to_string();

    // Create a database entry for the uploaded asset
    let result = insert_asset_into_database(
        &mut conn,
        namespace,
        filename.clone(),
        kind,
        asset_id,
        room,
        size,
    )
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

    Ok((asset_id, filename))
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
    kind: AssetFileKind,
    asset_id: AssetId,
    room: Room,
    size: i64,
) -> opentalk_database::Result<Asset> {
    NewAsset {
        id: asset_id,
        namespace,
        filename,
        kind: kind.to_string(),
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

    if let Some(max_storage) = user_tariff.quota(&QuotaType::MaxStorage) {
        if used_storage > BigDecimal::from(max_storage) {
            return AssetStorageExceededSnafu.fail();
        }
    }

    Ok(())
}

#[cfg(test)]
mod tests {
    use std::str::FromStr as _;

    use chrono::{TimeZone as _, Utc};
    use opentalk_types_common::{assets::FileExtension, time::Timestamp};
    use pretty_assertions::assert_eq;

    use super::{AssetFileKind, NewAssetFileName};

    #[test]
    fn new_asset_filename() {
        let timestamp = Timestamp::from(Utc.with_ymd_and_hms(2020, 5, 3, 14, 16, 19).unwrap());

        let filename = NewAssetFileName::new(
            AssetFileKind::from_str("recording").unwrap(),
            timestamp,
            FileExtension::from_str("mkv").unwrap(),
        );
        assert_eq!(
            "recording_2020-05-03_14-16-19-UTC.mkv",
            &filename.to_string()
        );

        let filename = NewAssetFileName::new_with_event_title(
            Some("A very (!!1~) Special Event!".to_string()),
            AssetFileKind::from_str("meetingnotes_pdf").unwrap(),
            timestamp,
            FileExtension::pdf(),
        );
        assert_eq!(
            "A very ___1__ Special Event__meetingnotes_pdf_2020-05-03_14-16-19-UTC.pdf",
            &filename.to_string()
        );

        let filename = NewAssetFileName::new_with_event_title(
            Some("世界您好".to_string()),
            AssetFileKind::from_str("meetingnotes_pdf").unwrap(),
            timestamp,
            FileExtension::pdf(),
        );
        assert_eq!(
            "世界您好_meetingnotes_pdf_2020-05-03_14-16-19-UTC.pdf",
            &filename.to_string()
        );
    }

    #[test]
    fn new_asset_file_kind() {
        // Too short
        assert!(AssetFileKind::from_str("").is_err());

        // Minimum length
        assert!(AssetFileKind::from_str("a").is_ok());

        // Maximum length
        assert!(AssetFileKind::from_str("abcdefghijabcdefghij").is_ok());

        // Too long
        assert!(AssetFileKind::from_str("abcdefghijabcdefghijk").is_err());

        // Valid characters
        assert!(AssetFileKind::from_str("bcdef1235489").is_ok());
        assert!(AssetFileKind::from_str("1337hello").is_ok());

        // Invalid characters
        assert!(AssetFileKind::from_str("a.").is_err());
        assert!(AssetFileKind::from_str("a!").is_err());
        assert!(AssetFileKind::from_str("a-").is_err());
        assert!(AssetFileKind::from_str("a?").is_err());
        assert!(AssetFileKind::from_str("a/").is_err());
    }
}
