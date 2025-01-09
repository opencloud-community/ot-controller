// SPDX-FileCopyrightText: OpenTalk GmbH <mail@opentalk.eu>
//
// SPDX-License-Identifier: EUPL-1.2

use actix_http::StatusCode;
use actix_web::{
    delete, get,
    web::{Data, Path, Query},
    HttpResponse,
};
use opentalk_controller_utils::CaptureApiError;
use opentalk_database::Db;
use opentalk_db_storage::assets::Asset;
use opentalk_signaling_core::{
    assets::{delete_asset, get_asset},
    ObjectStorage,
};
use opentalk_types_api_v1::{
    assets::AssetResource, error::ApiError, pagination::PagePaginationQuery,
    rooms::by_room_id::assets::RoomsByRoomIdAssetsGetResponseBody,
};
use opentalk_types_common::{assets::AssetId, rooms::RoomId};

use super::{response::NoContent, ApiResponse};
use crate::api::responses::{BinaryData, Forbidden, InternalServerError, NotFound, Unauthorized};

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
            body = RoomsByRoomIdAssetsGetResponseBody,
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
) -> Result<ApiResponse<RoomsByRoomIdAssetsGetResponseBody>, ApiError> {
    Ok(room_assets_inner(&db, room_id.into_inner(), pagination.into_inner()).await?)
}

async fn room_assets_inner(
    db: &Db,
    room_id: RoomId,
    PagePaginationQuery { per_page, page }: PagePaginationQuery,
) -> Result<ApiResponse<RoomsByRoomIdAssetsGetResponseBody>, CaptureApiError> {
    let mut conn = db.get_conn().await?;

    let (assets, asset_count) =
        Asset::get_all_for_room_paginated(&mut conn, room_id, per_page, page).await?;

    let asset_data = assets.into_iter().map(asset_to_asset_resource).collect();

    Ok(
        ApiResponse::new(RoomsByRoomIdAssetsGetResponseBody(asset_data)).with_page_pagination(
            per_page,
            page,
            asset_count,
        ),
    )
}

/// Get a specific asset inside a room.
///
/// This will return the plain asset contents, e.g. the binary file contents or
/// whatever else is stored inside the asset storage.
#[utoipa::path(
    params(
        ("room_id" = RoomId, description = "The id of the room"),
        ("asset_id" = AssetId, description = "The id of the asset"),
    ),
    responses(
        (
            status = StatusCode::OK,
            response = BinaryData,
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
#[get("/rooms/{room_id}/assets/{asset_id}")]
pub async fn room_asset(
    db: Data<Db>,
    storage: Data<ObjectStorage>,
    path: Path<(RoomId, AssetId)>,
) -> Result<HttpResponse, ApiError> {
    let (room_id, asset_id) = path.into_inner();
    Ok(room_asset_inner(&db, &storage, room_id, asset_id).await?)
}

async fn room_asset_inner(
    db: &Db,
    storage: &ObjectStorage,
    room_id: RoomId,
    asset_id: AssetId,
) -> Result<HttpResponse, CaptureApiError> {
    let asset = Asset::get(&mut db.get_conn().await?, asset_id, room_id).await?;

    let data = get_asset(storage, &asset.id).await?;

    Ok(HttpResponse::build(StatusCode::OK).streaming(data))
}

/// Delete an asset from a room.
///
/// The asset is removed from the room and deleted from the storage.
#[utoipa::path(
    operation_id = "delete_room_asset",
    params(
        ("room_id" = RoomId, description = "The id of the room"),
        ("asset_id" = AssetId, description = "The id of the asset"),
    ),
    responses(
        (
            status = StatusCode::NO_CONTENT,
            description = "The asset has been deleted",
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
#[delete("/rooms/{room_id}/assets/{asset_id}")]
pub async fn delete(
    db: Data<Db>,
    storage: Data<ObjectStorage>,
    path: Path<(RoomId, AssetId)>,
) -> Result<NoContent, ApiError> {
    Ok(delete_inner(&db, &storage, path.into_inner()).await?)
}

async fn delete_inner(
    db: &Db,
    storage: &ObjectStorage,
    (room_id, asset_id): (RoomId, AssetId),
) -> Result<NoContent, CaptureApiError> {
    delete_asset(storage, db, room_id, asset_id).await?;

    Ok(NoContent)
}

pub(crate) fn asset_to_asset_resource(asset: Asset) -> AssetResource {
    let Asset {
        id,
        created_at,
        updated_at: _,
        namespace,
        kind,
        filename,
        tenant_id: _,
        size,
    } = asset;
    AssetResource {
        id,
        filename,
        namespace,
        created_at,
        kind,
        size,
    }
}
