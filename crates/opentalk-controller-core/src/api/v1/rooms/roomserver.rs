// SPDX-FileCopyrightText: OpenTalk GmbH <mail@opentalk.eu>
//
// SPDX-License-Identifier: EUPL-1.2

//! Roomserver related API structs and Endpoints
//!
//! The defined structs are exposed to the REST API and will be serialized/deserialized. Similar
//! structs are defined in the Database crate [`opentalk_db_storage`] for database operations.

use actix_web::{
    post,
    web::{Data, Json, Path, ReqData},
};
use opentalk_controller_service::controller_backend::rooms::start_room_error::StartRoomError;
use opentalk_controller_service_facade::{OpenTalkControllerService, RequestUser};
use opentalk_types_api_v1::{
    error::{ApiError, ErrorBody},
    rooms::by_room_id::{
        PostRoomsRoomserverStartInvitedRequestBody, PostRoomsRoomserverStartRequestBody,
        RoomserverStartResponseBody,
    },
};
use opentalk_types_common::rooms::RoomId;

use crate::api::v1::rooms::InternalServerError;

/// Start a signaling session with the roomserver as a registered user
///
/// This endpoint has to be called in order to get a signaling token for the roomserver. This endpoint returns the token
/// and the corresponding roomserver address. Call the  *GET `<roomserver_address>/signaling/<token>`* endpoint to
/// establish the websocket connection with the roomserver.
#[utoipa::path(
    params(
        ("room_id" = RoomId, description = "The id of the room"),
    ),
    responses(
        (
            status = StatusCode::OK,
            description = "Returns the roomserver token and roomserver address",
            body = RoomserverStartResponseBody,
        ),
        (
            status = StatusCode::BAD_REQUEST,
            description = r"Returned when calling this endpoint on a controller where no roomserver is configured",
            body = ErrorBody,
            examples(
                (
                    "RoomserverSignalingDisabled" = (
                        summary = "Roomserver signaling is disabled", value = json!(ApiError::from(StartRoomError::RoomserverSignalingDisabled).body)
                    )
                ),
            ),
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
            status = StatusCode::FORBIDDEN,
            description = "The user has not been invited to join the room or has been banned from entering this room",
            body = ErrorBody,
            examples(
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
#[post("/rooms/{room_id}/roomserver/start")]
pub async fn start(
    service: Data<OpenTalkControllerService>,
    current_user: ReqData<RequestUser>,
    room_id: Path<RoomId>,
    request: Json<PostRoomsRoomserverStartRequestBody>,
) -> Result<Json<RoomserverStartResponseBody>, ApiError> {
    let response = Json(
        service
            .start_roomserver_room_session(
                current_user.into_inner(),
                room_id.into_inner(),
                request.into_inner(),
            )
            .await?,
    );

    Ok(response)
}

/// Start a signaling session with the roomserver with an invitation code
///
/// This endpoint has to be called in order to get a signaling token for the roomserver. This endpoint returns the token
/// and the corresponding roomserver address. Call the  *GET `<roomserver_address>/signaling/<token>`* endpoint to
/// establish the websocket connection with the roomserver.
#[utoipa::path(
    params(
        ("room_id" = RoomId, description = "The id of the room"),
    ),
    request_body = PostRoomsRoomserverStartInvitedRequestBody,
    responses(
        (
            status = StatusCode::OK,
            description = "Returns the roomserver token and roomserver address",
            body = RoomserverStartResponseBody,
        ),
        (
            status = StatusCode::BAD_REQUEST,
            description = r"The provided ID token is malformed or contains
                invalid claims,  no breakout rooms were found for this room, the
                breakout room id is invalid, the room doesn't exist, the guest
                does not have a valid invite for this room or when calling this
                endpoint on a controller where no roomserver is configured. Guests
                shall not be able to distinguish between existing rooms and rooms 
                they don't have permission to enter, therefore the response is the
                same in these cases.",
            body = ErrorBody,
            examples(
                (
                    "RoomIdMismatch" = (
                        summary = "Room id mismatch", value = json!(ErrorBody::new("bad_request", "Room id mismatch"))
                    )
                ),
                (
                    "RoomserverSignalingDisabled" = (
                        summary = "Roomserver signaling is disabled", value = json!(ApiError::from(StartRoomError::RoomserverSignalingDisabled).body)
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
            description = "The specified room could not be found or it has no event associated with it",
            body = ErrorBody,
            example = json!(ApiError::not_found().body),
        ),
        (
            status = StatusCode::INTERNAL_SERVER_ERROR,
            response = InternalServerError,
        ),
    ),
    security(),
)]
#[post("/rooms/{room_id}/roomserver/start_invited")]
pub async fn start_invited(
    service: Data<OpenTalkControllerService>,
    room_id: Path<RoomId>,
    request: Json<PostRoomsRoomserverStartInvitedRequestBody>,
) -> Result<Json<RoomserverStartResponseBody>, ApiError> {
    let response = Json(
        service
            .start_invited_roomserver_room_session(room_id.into_inner(), request.into_inner())
            .await?,
    );

    Ok(response)
}
