// SPDX-FileCopyrightText: OpenTalk GmbH <mail@opentalk.eu>
//
// SPDX-License-Identifier: EUPL-1.2

//! Contains invite related REST endpoints.
use actix_web::{
    delete, get, post, put,
    web::{Data, Json, Path, Query, ReqData},
};
use opentalk_controller_service_facade::{OpenTalkControllerService, RequestUser};
use opentalk_types_api_v1::{
    error::ApiError,
    pagination::PagePaginationQuery,
    rooms::by_room_id::invites::{
        GetRoomsInvitesResponseBody, InviteResource, PostInviteRequestBody,
        PostInviteVerifyRequestBody, PostInviteVerifyResponseBody, PutInviteRequestBody,
        RoomIdAndInviteCode,
    },
};
use opentalk_types_common::rooms::RoomId;

use super::{DefaultApiResult, response::NoContent};
use crate::api::{
    responses::{Forbidden, InternalServerError, NotFound, Unauthorized},
    v1::ApiResponse,
};

/// Create a new invite
///
/// A new invite to the room is created with the information in the body.
#[utoipa::path(
    params(
        ("room_id" = RoomId, description = "The id of the room"),
    ),
    request_body = PostInviteRequestBody,
    responses(
        (
            status = StatusCode::OK,
            description = "Successfully create a new invite",
            body = InviteResource,
        ),
        (
            status = StatusCode::BAD_REQUEST,
            description = "Could not create a new invite due to wrong syntax or
                bad values, for example an invalid owner id.",
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
#[post("/rooms/{room_id}/invites")]
pub async fn add_invite(
    service: Data<OpenTalkControllerService>,
    current_user: ReqData<RequestUser>,
    room_id: Path<RoomId>,
    new_invite: Json<PostInviteRequestBody>,
) -> DefaultApiResult<InviteResource> {
    let current_user = current_user.into_inner();
    let room_id = room_id.into_inner();
    let new_invite = new_invite.into_inner();

    let invite_resource = service
        .create_invite(current_user, room_id, new_invite)
        .await?;

    Ok(ApiResponse::new(invite_resource))
}

/// Get all invites for a room
///
/// This returns all invites that are available for a room. If no
/// pagination query is added, the default page size is used.
#[utoipa::path(
    params(
        ("room_id" = RoomId, description = "The id of the room"),
        PagePaginationQuery,
    ),
    responses(
        (
            status = StatusCode::OK,
            description = "The invites could be loaded successfully",
            body = GetRoomsInvitesResponseBody,
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
#[get("/rooms/{room_id}/invites")]
pub async fn get_invites(
    service: Data<OpenTalkControllerService>,
    room_id: Path<RoomId>,
    pagination: Query<PagePaginationQuery>,
) -> DefaultApiResult<GetRoomsInvitesResponseBody> {
    let room_id = room_id.into_inner();

    let (invite_resources, invite_count) = service.get_invites(room_id, &pagination).await?;

    Ok(ApiResponse::new(invite_resources).with_page_pagination(
        pagination.per_page,
        pagination.page,
        invite_count,
    ))
}

/// Get a room invite
///
/// Returns the room invite resource
#[utoipa::path(
    params(RoomIdAndInviteCode),
    responses(
        (
            status = StatusCode::OK,
            description = "Successfully retrieved the room invite",
            body = InviteResource,
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
#[get("/rooms/{room_id}/invites/{invite_code}")]
pub async fn get_invite(
    service: Data<OpenTalkControllerService>,
    path_params: Path<RoomIdAndInviteCode>,
) -> DefaultApiResult<InviteResource> {
    let invite_resoruce = service
        .get_invite(path_params.room_id, path_params.invite_code)
        .await?;

    Ok(ApiResponse::new(invite_resoruce))
}

/// Update an invite code
///
/// Updates the field values as set in the request body.
#[utoipa::path(
    params(RoomIdAndInviteCode),
    request_body = PutInviteRequestBody,
    responses(
        (
            status = StatusCode::OK,
            description = "Successfully updated the room invite",
            body = InviteResource,
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
#[put("/rooms/{room_id}/invites/{invite_code}")]
pub async fn update_invite(
    service: Data<OpenTalkControllerService>,
    current_user: ReqData<RequestUser>,
    path_params: Path<RoomIdAndInviteCode>,
    update_invite: Json<PutInviteRequestBody>,
) -> DefaultApiResult<InviteResource> {
    let current_user = current_user.into_inner();

    let invite_resource = service
        .update_invite(
            current_user,
            path_params.room_id,
            path_params.invite_code,
            update_invite.into_inner(),
        )
        .await?;

    Ok(ApiResponse::new(invite_resource))
}

/// Delete an invite code
///
/// The invite code will no longer be usable once it is deleted.
#[utoipa::path(
    params(RoomIdAndInviteCode),
    responses(
        (
            status = StatusCode::NO_CONTENT,
            description = "Successfully deleted the room invite",
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
#[delete("/rooms/{room_id}/invites/{invite_code}")]
pub async fn delete_invite(
    service: Data<OpenTalkControllerService>,
    current_user: ReqData<RequestUser>,
    path_params: Path<RoomIdAndInviteCode>,
) -> Result<NoContent, ApiError> {
    let current_user = current_user.into_inner();

    service
        .delete_invite(current_user, path_params.room_id, path_params.invite_code)
        .await?;

    Ok(NoContent)
}

/// Verify an invite code
///
/// Verifies the invite and returns the room url for the invite code
#[utoipa::path(
    request_body = PostInviteVerifyRequestBody,
    responses(
        (
            status = StatusCode::OK,
            description = "Invite is valid, the response body tells the room id",
            body = PostInviteVerifyResponseBody,
        ),
        (
            status = StatusCode::UNAUTHORIZED,
            response = Unauthorized,
        ),
        (
            status = StatusCode::NOT_FOUND,
            response = NotFound,
        ),
        (
            status = StatusCode::UNPROCESSABLE_ENTITY,
            description = "Invalid body contents received",
        ),
        (
            status = StatusCode::INTERNAL_SERVER_ERROR,
            response = InternalServerError,
        ),
    ),
    security(),
)]
#[post("/invite/verify")]
pub async fn verify_invite_code(
    service: Data<OpenTalkControllerService>,
    verify_request: Json<PostInviteVerifyRequestBody>,
) -> DefaultApiResult<PostInviteVerifyResponseBody> {
    let verify_request = verify_request.into_inner();

    let verify_response = service.verify_invite_code(verify_request).await?;

    Ok(ApiResponse::new(verify_response))
}
