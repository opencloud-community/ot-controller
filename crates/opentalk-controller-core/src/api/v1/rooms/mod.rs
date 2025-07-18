// SPDX-FileCopyrightText: OpenTalk GmbH <mail@opentalk.eu>
//
// SPDX-License-Identifier: EUPL-1.2

//! Room related API structs and Endpoints
//!
//! The defined structs are exposed to the REST API and will be serialized/deserialized. Similar
//! structs are defined in the Database crate [`opentalk_db_storage`] for database operations.

use actix_web::{
    delete, get, patch, post,
    web::{self, Data, Json, Path, ReqData},
};
use opentalk_controller_service::controller_backend::rooms::start_room_error::StartRoomError;
use opentalk_controller_service_facade::{OpenTalkControllerService, RequestUser};
use opentalk_db_storage::users::User;
use opentalk_types_api_v1::{
    error::{ApiError, ErrorBody},
    pagination::PagePaginationQuery,
    rooms::{
        GetRoomsResponseBody, PostRoomsRequestBody, RoomResource,
        by_room_id::{
            DeleteRoomQuery, GetRoomEventResponseBody, PatchRoomsRequestBody,
            PostRoomsStartInvitedRequestBody, PostRoomsStartRequestBody, RoomsStartResponseBody,
        },
    },
};
use opentalk_types_common::{events::EventInfo, rooms::RoomId, tariffs::TariffResource};

use super::response::NoContent;
use crate::api::{
    headers::PageLink,
    responses::{Forbidden, InternalServerError, NotFound, Unauthorized},
    v1::ApiResponse,
};

pub(crate) mod roomserver;

/// Get a list of rooms accessible by the requesting user
///
/// All rooms accessible to the requesting user are returned in a list. If no
/// pagination query is added, the default page size is used.
#[utoipa::path(
    params(PagePaginationQuery),
    responses(
        (
            status = StatusCode::OK,
            description = "List of accessible rooms successfully returned",
            body = GetRoomsResponseBody,
            headers(
                ("link" = PageLink, description = "Links for paging through the results"),
            ),
        ),
        (
            status = StatusCode::UNAUTHORIZED,
            response = Unauthorized,
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
#[get("/rooms")]
pub async fn accessible(
    service: Data<OpenTalkControllerService>,
    current_user: ReqData<User>,
    pagination: web::Query<PagePaginationQuery>,
) -> Result<ApiResponse<GetRoomsResponseBody>, ApiError> {
    let current_user = current_user.into_inner();
    let pagination = pagination.into_inner();

    let (rooms, room_count) = service.get_rooms(current_user.id, &pagination).await?;

    Ok(ApiResponse::new(rooms).with_page_pagination(
        pagination.per_page,
        pagination.page,
        room_count,
    ))
}

/// Create a new room
///
/// Creates a new room withh the settings given in the request body.
#[utoipa::path(
    request_body = PostRoomsRequestBody,
    responses(
        (
            status = StatusCode::CREATED,
            description = "Room successfully created",
            body = RoomResource,
        ),
        (
            status = StatusCode::BAD_REQUEST,
            description = "Wrong syntax or bad values such as invalid owner id received in the body",
        ),
        (
            status = StatusCode::UNAUTHORIZED,
            response = Unauthorized,
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
    security(
        ("BearerAuth" = []),
    ),
)]
#[post("/rooms")]
pub async fn new(
    service: Data<OpenTalkControllerService>,
    current_user: ReqData<RequestUser>,
    body: Json<PostRoomsRequestBody>,
) -> Result<Json<RoomResource>, ApiError> {
    let current_user = current_user.into_inner();
    let body = body.into_inner();

    let room_resource = service
        .create_room(
            current_user,
            body.password,
            body.enable_sip,
            body.waiting_room,
            body.e2e_encryption,
        )
        .await?;

    Ok(Json(room_resource))
}

/// Patch a room with the provided fields
///
/// Fields that are not provided in the request body will remain unchanged.
#[utoipa::path(
    request_body = PatchRoomsRequestBody,
    operation_id = "patch_room",
    params(
        ("room_id" = RoomId, description = "The id of the room to be modified"),
    ),
    responses(
        (
            status = StatusCode::OK,
            description = "Room was successfully updated",
            body = RoomResource
        ),
        (
            status = StatusCode::BAD_REQUEST,
            description = r"Could not modify the specified room due to wrong
                syntax or bad values, for example an invalid owner id",
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
            status = StatusCode::INTERNAL_SERVER_ERROR,
            response = InternalServerError,
        ),
    ),
    security(
        ("BearerAuth" = []),
    ),
)]
#[patch("/rooms/{room_id}")]
pub async fn patch(
    service: Data<OpenTalkControllerService>,
    current_user: ReqData<RequestUser>,
    room_id: Path<RoomId>,
    body: Json<PatchRoomsRequestBody>,
) -> Result<Json<RoomResource>, ApiError> {
    let current_user = current_user.into_inner();
    let room_id = room_id.into_inner();
    let body = body.into_inner();

    let room_resource = service
        .patch_room(
            current_user,
            room_id,
            body.password,
            body.waiting_room,
            body.e2e_encryption,
        )
        .await?;

    Ok(Json(room_resource))
}

/// Delete a room and its owned resources.
///
/// Deletes the room by the id if found. See the query parameters for affecting
/// the behavior of this endpoint, such as succeding even if external resources
/// cannot be successfully deleted.
#[utoipa::path(
    params(
        ("room_id" = RoomId, description = "The id of the room"),
        DeleteRoomQuery,
    ),
    responses(
        (
            status = StatusCode::NO_CONTENT,
            description = "Room was successfully deleted",
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
#[delete("/rooms/{room_id}")]
pub async fn delete(
    service: Data<OpenTalkControllerService>,
    current_user: ReqData<RequestUser>,
    room_id: Path<RoomId>,
    query: web::Query<DeleteRoomQuery>,
) -> Result<NoContent, ApiError> {
    let query = query.into_inner();

    service
        .delete_room(
            current_user.into_inner(),
            room_id.into_inner(),
            query.force_delete_reference_if_external_services_fail,
        )
        .await?;

    Ok(NoContent)
}

/// Get a room
///
/// Returns the room resource including additional information such as the creator profile.
#[utoipa::path(
    params(
        ("room_id" = RoomId, description = "The id of the room"),
    ),
    responses(
        (
            status = StatusCode::OK,
            description = "Room was successfully retrieved",
            body = RoomResource
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
#[get("/rooms/{room_id}")]
pub async fn get(
    service: Data<OpenTalkControllerService>,
    room_id: Path<RoomId>,
) -> Result<Json<RoomResource>, ApiError> {
    Ok(Json(service.get_room(&room_id).await?))
}

/// Get a room's tariff
///
/// This returns the tariff that applies to the room, typically the tariff of
/// the room creator.
#[utoipa::path(
    params(
        ("room_id" = RoomId, description = "The id of the room"),
    ),
    responses(
        (
            status = StatusCode::OK,
            description = "The room's tariff was successfully retrieved",
            body = TariffResource
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
        ("InviteCode" = []),
    ),
)]
#[get("/rooms/{room_id}/tariff")]
pub async fn get_room_tariff(
    service: Data<OpenTalkControllerService>,
    room_id: Path<RoomId>,
) -> Result<Json<TariffResource>, ApiError> {
    Ok(Json(service.get_room_tariff(&room_id).await?))
}

/// Get a room's event
///
/// This returns the event with which the room is associated. Please note
/// that rooms can exist without events, in which case a `404` status will be
/// returned.
#[utoipa::path(
    params(
        ("room_id" = RoomId, description = "The id of the room"),
    ),
    responses(
        (
            status = StatusCode::OK,
            description = "The room's event was successfully retrieved",
            body = EventInfo
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
        ("InviteCode" = []),
    ),
)]
#[get("/rooms/{room_id}/event")]
pub async fn get_room_event(
    service: Data<OpenTalkControllerService>,
    room_id: Path<RoomId>,
) -> Result<Json<GetRoomEventResponseBody>, ApiError> {
    Ok(Json(service.get_room_event(&room_id).await?))
}

/// Start a signaling session as a registered user
///
/// This endpoint has to be called in order to get a room ticket. When joining a room, the ticket
/// must be provided as a `Sec-WebSocket-Protocol` header field when starting the WebSocket
/// connection.
#[utoipa::path(
    params(
        ("room_id" = RoomId, description = "The id of the room"),
    ),
    responses(
        (
            status = StatusCode::OK,
            description = "Returns the information for joining the room",
            body = RoomsStartResponseBody,
        ),
        (
            status = StatusCode::UNAUTHORIZED,
            description = r"The provided AccessToken is expired or the
                provided ID- or Access-Token is invalid. The WWW-Authenticate
                header will contain a error description 'session expired' to
                distinguish between an invalid and an expired token.",
            body = ErrorBody,
            headers(
                (
                    "www-authenticate",
                    description = "Will contain 'session expired' to distinguish between an invalid and an expired token"
                ),
            ),
        ),
        (
            status = StatusCode::BAD_REQUEST,
            description = "Either no breakout rooms were found for this room, the breakout room id is invalid or legacy signaling is disabled for this controller",
            body = ErrorBody,
            examples(
                ("NoBreakoutRooms" = (summary = "No breakout rooms", value = json!(ApiError::from(StartRoomError::NoBreakoutRooms).body))),
                ("InvalidBreakoutRoomId" = (summary = "Invalid breakout room id", value = json!(ApiError::from(StartRoomError::InvalidBreakoutRoomId).body))),
                ("LegacySignalingDisabled" = (summary = "Legacy signaling is disabled", value = json!(ApiError::from(StartRoomError::LegacySignalingDisabled).body))) 
            ),
        ),
        (
            status = StatusCode::FORBIDDEN,
            description = "The user has not been invited to join the room or has been banned from entering this room",
            body = ErrorBody,
            examples(
                ("UserBanned" = (summary = "User has been banned from the room", value = json!(ApiError::from(StartRoomError::BannedFromRoom).body))),
                ("UserNotInvited" = (summary = "User has not been invited to join the room", value = json!(ApiError::forbidden().body))),
            ),
        ),
        (
            status = StatusCode::NOT_FOUND,
            description = "The specified room could not be found or it has no event associated with it",
            body = ErrorBody,
            example = json!(ApiError::not_found().body),
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
#[post("/rooms/{room_id}/start")]
pub async fn start(
    service: Data<OpenTalkControllerService>,
    current_user: ReqData<RequestUser>,
    room_id: Path<RoomId>,
    request: Json<PostRoomsStartRequestBody>,
) -> Result<Json<RoomsStartResponseBody>, ApiError> {
    let response = Json(
        service
            .start_room_session(
                current_user.into_inner(),
                room_id.into_inner(),
                request.into_inner(),
            )
            .await?,
    );

    Ok(response)
}

/// Start a signaling session for an invitation code
///
/// Returns a ticket to be used with the `/signaling` endpoint. When joining a
/// room, the ticket must be provided as `Sec-WebSocket-Protocol` header field
/// when starting the WebSocket connection. When the requested room has a
/// password set, the requester must provide the correct password through the
/// requests body. When the request has no password set, the password will be
/// ignored if provided.
#[utoipa::path(
    params(
        ("room_id" = RoomId, description = "The id of the room"),
    ),
    request_body = PostRoomsStartInvitedRequestBody,
    responses(
        (
            status = StatusCode::OK,
            description = "Response body includes the information needed to connect to the signaling endpoint",
            body = RoomsStartResponseBody,
        ),
        (
            status = StatusCode::BAD_REQUEST,
            description = r"The provided ID token is malformed or contains
                invalid claims,  no breakout rooms were found for this room, the
                breakout room id is invalid, the room doesn't exist, the guest
                does not have a valid invite for this room or legacy signaling has been
                disabled for this controller. Guests shall not be able to distinguish
                between existing rooms and rooms they don't have permission to enter,
                therefore the response is the same in these cases",
            body = ErrorBody,
            examples(
                (
                    "NoBreakoutRooms" = (
                        summary = "No breakout rooms", value = json!(ApiError::from(StartRoomError::NoBreakoutRooms).body)
                    )
                ),
                (
                    "InvalidBreakoutRoomId" = (
                        summary = "Invalid breakout room id", value = json!(ApiError::from(StartRoomError::InvalidBreakoutRoomId).body)
                    )
                ),
                (
                    "LegacySignalingDisabled" = (
                        summary = "Legacy signaling is disabled", value = json!(ApiError::from(StartRoomError::LegacySignalingDisabled).body)
                    )
                ),
                (
                    "RoomIdMismatch" = (
                        summary = "Room id mismatch", value = json!(ErrorBody::new("bad_request", "Room id mismatch"))
                    )
                ),
            ),
        ),
        (
            status = StatusCode::UNPROCESSABLE_ENTITY,
            description = "Invalid invite code",
        ),
        (
            status = StatusCode::UNPROCESSABLE_ENTITY,
            description = "Invalid body contents received",
        ),
        (
            status = StatusCode::UNAUTHORIZED,
            body = ErrorBody,
            description = r"Either: the provided access token is expired or the
                provided id or access token is invalid. The WWW-Authenticate
                header will contain an error description 'session expired' to
                distinguish between an invalid and an expired token.
                Or: the provided password was incorrect, in which case the body
                contains more information.",
            headers(
                (
                    "www-authenticate",
                    description = "Will contain 'session expired' to distinguish between an invalid and an expired token"
                ),
            ),
            examples(
                ("WrongRoomPassword" = (
                    summary = "Wrong room password",
                    value = json!(ApiError::from(StartRoomError::WrongRoomPassword).body)
                )),
                ("ExpiredOrInvalidAccessToken" = (
                    summary = "Expired or invalid access token",
                    value = json!(
                        ApiError::unauthorized()
                        .with_message("The session for this user has expired")
                        .with_www_authenticate(opentalk_types_api_v1::error::AuthenticationError::SessionExpired)
                        .body
                    )
                )),
            ),
        ),
        (
            status = StatusCode::FORBIDDEN,
            body = ErrorBody,
            description = "The participant has been banned from entering this room",
            example = json!(ApiError::from(StartRoomError::BannedFromRoom).body),
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
    security(),
)]
#[post("/rooms/{room_id}/start_invited")]
pub async fn start_invited(
    service: Data<OpenTalkControllerService>,
    room_id: Path<RoomId>,
    request: Json<PostRoomsStartInvitedRequestBody>,
) -> Result<ApiResponse<RoomsStartResponseBody>, ApiError> {
    let response = service
        .start_invited_room_session(room_id.into_inner(), request.into_inner())
        .await?;

    Ok(ApiResponse::new(response))
}
