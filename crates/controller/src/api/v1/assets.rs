// SPDX-FileCopyrightText: OpenTalk GmbH <mail@opentalk.eu>
//
// SPDX-License-Identifier: EUPL-1.2

use super::response::{ApiError, NoContent};
use super::{ApiResponse, PagePaginationQuery};
use actix_http::StatusCode;
use actix_web::web::{Data, Path, Query};
use actix_web::{delete, get, HttpResponse};
use chrono::{DateTime, Utc};
use database::Db;
use db_storage::assets::Asset;
use serde::Serialize;
use signaling_core::{
    assets::{delete_asset, get_asset},
    ObjectStorage,
};
use types::core::{AssetId, RoomId};

#[derive(Debug, Serialize)]
pub struct AssetResource {
    id: AssetId,
    filename: String,
    #[serde(skip_serializing_if = "Option::is_none")]
    namespace: Option<String>,
    created_at: DateTime<Utc>,
}

impl From<Asset> for AssetResource {
    fn from(asset: Asset) -> Self {
        AssetResource {
            id: asset.id,
            filename: asset.filename,
            namespace: asset.namespace,
            created_at: asset.created_at,
        }
    }
}

#[get("/rooms/{room_id}/assets")]
pub async fn room_assets(
    db: Data<Db>,
    room_id: Path<RoomId>,
    pagination: Query<PagePaginationQuery>,
) -> Result<ApiResponse<Vec<AssetResource>>, ApiError> {
    let room_id = room_id.into_inner();
    let PagePaginationQuery { per_page, page } = pagination.into_inner();

    let mut conn = db.get_conn().await?;

    let (assets, asset_count) =
        Asset::get_all_for_room_paginated(&mut conn, room_id, per_page, page).await?;

    let asset_data = assets.into_iter().map(Into::into).collect();

    Ok(ApiResponse::new(asset_data).with_page_pagination(per_page, page, asset_count))
}

#[get("/rooms/{room_id}/assets/{asset_id}")]
pub async fn room_asset(
    db: Data<Db>,
    storage: Data<ObjectStorage>,
    path: Path<(RoomId, AssetId)>,
) -> Result<HttpResponse, ApiError> {
    let (room_id, asset_id) = path.into_inner();

    let asset = Asset::get(&mut db.get_conn().await?, asset_id, room_id).await?;

    let data = get_asset(&storage, &asset.id).await?;

    Ok(HttpResponse::build(StatusCode::OK).streaming(data))
}

#[delete("/rooms/{room_id}/assets/{asset_id}")]
pub async fn delete(
    db: Data<Db>,
    storage: Data<ObjectStorage>,
    path: Path<(RoomId, AssetId)>,
) -> Result<NoContent, ApiError> {
    let (room_id, asset_id) = path.into_inner();

    delete_asset(&storage, db.into_inner(), room_id, asset_id).await?;

    Ok(NoContent)
}
