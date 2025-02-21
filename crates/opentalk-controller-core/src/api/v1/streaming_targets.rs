// SPDX-FileCopyrightText: OpenTalk GmbH <mail@opentalk.eu>
//
// SPDX-License-Identifier: EUPL-1.2

//! Contains invite related REST endpoints.
use actix_web::{
    delete, get, patch, post,
    web::{Data, Json, Path, Query, ReqData},
};
use opentalk_controller_service_facade::{OpenTalkControllerService, RequestUser};
use opentalk_types_api_v1::{
    error::ApiError,
    events::StreamingTargetOptionsQuery,
    pagination::PagePaginationQuery,
    rooms::by_room_id::streaming_targets::{
        GetRoomStreamingTargetResponseBody, GetRoomStreamingTargetsResponseBody,
        PatchRoomStreamingTargetRequestBody, PatchRoomStreamingTargetResponseBody,
        PostRoomStreamingTargetRequestBody, PostRoomStreamingTargetResponseBody,
        RoomAndStreamingTargetId,
    },
};
use opentalk_types_common::rooms::RoomId;

use super::{response::NoContent, DefaultApiResult};
use crate::api::{
    responses::{Forbidden, InternalServerError, NotFound, Unauthorized},
    v1::ApiResponse,
};

/// Lists the streaming targets of a room
///
/// Returns the streaming targets available for a room
#[utoipa::path(
    params(
        PagePaginationQuery,
        ("room_id" = RoomId, description = "The id of the room"),
    ),
    responses(
        (
            status = StatusCode::OK,
            description = "List of streaming targets successfully returned",
            body = GetRoomStreamingTargetsResponseBody,
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
#[get("/rooms/{room_id}/streaming_targets")]
pub async fn get_streaming_targets(
    service: Data<OpenTalkControllerService>,
    current_user: ReqData<RequestUser>,
    room_id: Path<RoomId>,
    pagination: Query<PagePaginationQuery>,
) -> DefaultApiResult<GetRoomStreamingTargetsResponseBody> {
    let response = service
        .get_streaming_targets(current_user.id, room_id.into_inner(), &pagination)
        .await?;
    let length = response.0.len();

    Ok(ApiResponse::new(response).with_page_pagination(
        pagination.per_page,
        pagination.page,
        length as i64,
    ))
}

/// Creates a new streaming target
///
/// Creates a new streaming target for the given room
#[utoipa::path(
    params(
        StreamingTargetOptionsQuery,
        ("room_id" = RoomId, description = "The id of the room"),
    ),
    request_body = PostRoomStreamingTargetRequestBody,
    responses(
        (
            status = StatusCode::OK,
            description = "Successfully create a new streaming target",
            body = PostRoomStreamingTargetResponseBody,
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
#[post("/rooms/{room_id}/streaming_targets")]
#[allow(clippy::too_many_arguments)]
pub async fn post_streaming_target(
    service: Data<OpenTalkControllerService>,
    current_user: ReqData<RequestUser>,
    room_id: Path<RoomId>,
    query: Query<StreamingTargetOptionsQuery>,
    data: Json<PostRoomStreamingTargetRequestBody>,
) -> DefaultApiResult<PostRoomStreamingTargetResponseBody> {
    let response = service
        .post_streaming_target(
            current_user.into_inner(),
            room_id.into_inner(),
            query.into_inner(),
            data.into_inner().0,
        )
        .await?;

    Ok(ApiResponse::new(response))
}

/// Gets a streaming target
///
/// Returns a single streaming target for a specific room.
#[utoipa::path(
    params(RoomAndStreamingTargetId),
    responses(
        (
            status = StatusCode::OK,
            description = "The streaming target has been successfully returned",
            body = GetRoomStreamingTargetResponseBody,
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
#[get("/rooms/{room_id}/streaming_targets/{streaming_target_id}")]
pub async fn get_streaming_target(
    service: Data<OpenTalkControllerService>,
    current_user: ReqData<RequestUser>,
    path_params: Path<RoomAndStreamingTargetId>,
) -> DefaultApiResult<GetRoomStreamingTargetResponseBody> {
    let response = service
        .get_streaming_target(current_user.id, path_params.into_inner())
        .await?;

    Ok(ApiResponse::new(response))
}

/// Updates a streaming target
///
/// Modifies and returns a single streaming target.
#[utoipa::path(
    params(RoomAndStreamingTargetId),
    request_body = PatchRoomStreamingTargetRequestBody,
    responses(
        (
            status = StatusCode::OK,
            description = "Streaming target was successfully updated",
            body = PatchRoomStreamingTargetResponseBody
        ),
        (
            status = StatusCode::BAD_REQUEST,
            description = r"Could not modify the specified streaming target due to wrong
                syntax or bad values",
        ),
        (
            status = StatusCode::UNAUTHORIZED,
            response = Unauthorized,
        ),
        (
            status = HttpStatus::FORBIDDEN,
            response = Forbidden,
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
#[patch("/rooms/{room_id}/streaming_targets/{streaming_target_id}")]
#[allow(clippy::too_many_arguments)]
pub async fn patch_streaming_target(
    service: Data<OpenTalkControllerService>,
    current_user: ReqData<RequestUser>,
    path_params: Path<RoomAndStreamingTargetId>,
    query: Query<StreamingTargetOptionsQuery>,
    streaming_target: Json<PatchRoomStreamingTargetRequestBody>,
) -> DefaultApiResult<PatchRoomStreamingTargetResponseBody> {
    let response = service
        .patch_streaming_target(
            current_user.into_inner(),
            path_params.into_inner(),
            query.into_inner(),
            streaming_target.into_inner(),
        )
        .await?;

    Ok(ApiResponse::new(response))
}

/// Deletes a streaming target
///
/// The streaming target is deleted from the room
#[utoipa::path(
    params(
        RoomAndStreamingTargetId,
        StreamingTargetOptionsQuery,
    ),
    responses(
        (
            status = StatusCode::NO_CONTENT,
            description = "The streaming target has been deleted",
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
#[delete("/rooms/{room_id}/streaming_targets/{streaming_target_id}")]
#[allow(clippy::too_many_arguments)]
pub async fn delete_streaming_target(
    service: Data<OpenTalkControllerService>,
    current_user: ReqData<RequestUser>,
    path_params: Path<RoomAndStreamingTargetId>,
    query: Query<StreamingTargetOptionsQuery>,
) -> Result<NoContent, ApiError> {
    service
        .delete_streaming_target(
            current_user.into_inner(),
            path_params.into_inner(),
            query.into_inner(),
        )
        .await?;

    Ok(NoContent)
}
