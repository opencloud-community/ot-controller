// SPDX-FileCopyrightText: OpenTalk GmbH <mail@opentalk.eu>
//
// SPDX-License-Identifier: EUPL-1.2

use actix_http::StatusCode;
use actix_web::{
    HttpResponse, delete, get, post,
    web::{Data, Path, Payload, Query},
};
use futures::TryStreamExt;
use opentalk_controller_service_facade::OpenTalkControllerService;
use opentalk_signaling_core::{ObjectStorageError, assets::NewAssetFileName};
use opentalk_types_api_v1::{
    error::ApiError,
    pagination::PagePaginationQuery,
    rooms::by_room_id::assets::{
        PostAssetQuery, PostAssetResponseBody, RoomsByRoomIdAssetsGetResponseBody,
    },
};
use opentalk_types_common::{assets::AssetId, rooms::RoomId, time::Timestamp};

use super::{ApiResponse, DefaultApiResult, response::NoContent};
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
    service: Data<OpenTalkControllerService>,
    room_id: Path<RoomId>,
    pagination: Query<PagePaginationQuery>,
) -> Result<ApiResponse<RoomsByRoomIdAssetsGetResponseBody>, ApiError> {
    let pagination = pagination.into_inner();

    let (assets, asset_count) = service
        .get_room_assets(room_id.into_inner(), &pagination)
        .await?;

    Ok(ApiResponse::new(assets).with_page_pagination(
        pagination.per_page,
        pagination.page,
        asset_count,
    ))
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
    service: Data<OpenTalkControllerService>,
    path: Path<(RoomId, AssetId)>,
) -> Result<HttpResponse, ApiError> {
    let (room_id, asset_id) = path.into_inner();

    let stream = service.get_room_asset(room_id, asset_id).await?;

    Ok(HttpResponse::build(StatusCode::OK).streaming(stream))
}

/// Create an asset for a room from an uploaded file
///
/// The asset is attached to the room and saved in the storage.
#[utoipa::path(
    operation_id = "create_room_asset",
    request_body(
        content = String,
        content_type = "application/octet-stream",
        description = "The contents of the file",
    ),
    params(
        ("room_id" = RoomId, description = "The id of the room"),
        PostAssetQuery,
    ),
    responses(
        (
            status = StatusCode::OK,
            description = "The asset has been created successfully",
            body = PostAssetResponseBody,
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
#[post("/rooms/{room_id}/assets")]
pub async fn create(
    service: Data<OpenTalkControllerService>,
    path: Path<RoomId>,
    query: Query<PostAssetQuery>,
    data: Payload,
) -> DefaultApiResult<PostAssetResponseBody, ApiError> {
    let room_id = path.into_inner();
    let query = query.into_inner();

    let filename = NewAssetFileName::new_with_event_title(
        query.event_title,
        query.kind,
        Timestamp::now(),
        query.file_extension,
    );

    let data = data.map_err(|e| ObjectStorageError::Other {
        message: "Upload error".to_string(),
        source: Some(e.into()),
    });

    let resource = service
        .create_room_asset(room_id, filename, query.namespace, Box::new(data))
        .await?;

    let response = PostAssetResponseBody(resource);

    Ok(ApiResponse::new(response))
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
    service: Data<OpenTalkControllerService>,
    path: Path<(RoomId, AssetId)>,
) -> Result<NoContent, ApiError> {
    let (room_id, asset_id) = path.into_inner();

    service.delete_room_asset(room_id, asset_id).await?;

    Ok(NoContent)
}
