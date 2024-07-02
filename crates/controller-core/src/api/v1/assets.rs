// SPDX-FileCopyrightText: OpenTalk GmbH <mail@opentalk.eu>
//
// SPDX-License-Identifier: EUPL-1.2

use actix_http::StatusCode;
use actix_web::{
    delete, get,
    web::{Data, Path, Query},
    HttpResponse,
};
use opentalk_database::Db;
use opentalk_db_storage::assets::Asset;
use opentalk_signaling_core::{
    assets::{delete_asset, get_asset},
    ObjectStorage,
};
use opentalk_types::{
    api::{
        error::ApiError,
        v1::{assets::AssetResource, pagination::PagePaginationQuery},
    },
    core::{AssetId, RoomId},
};

use super::{response::NoContent, ApiResponse};
use crate::api::responses::{Forbidden, InternalServerError, NotFound, Unauthorized};

/// Get the assets associated with a room.
///
/// This returns assets that are available for a room. If no
/// pagination query is added, the default page size is used.
#[utoipa::path(
    params(
        ("room_id" = RoomId, description = "The id of the room"),
        PagePaginationQuery,
    ),
    responses(
        (
            status = StatusCode::OK,
            description = "The assets have been returned successfully",
            body = GetRoomsAssetsResponseBody,
        ),
        (
            status = StatusCode::UNAUTHORIZED,
            response = Unauthorized,
        ),
        (
            status = StatusCode::FORBIDDEN,
            response = Forbidden,
        ),
        (
            status = StatusCode::NOT_FOUND,
            response = NotFound,
        ),
        (
            status = StatusCode::INTERNAL_SERVER_ERROR,
            response = InternalServerError,
        ),
    ),
    security(
        ("BearerAuth" = []),
    ),
)]
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
