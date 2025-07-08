// SPDX-FileCopyrightText: OpenTalk GmbH <mail@opentalk.eu>
//
// SPDX-License-Identifier: EUPL-1.2

use actix_web::{
    HttpResponse, delete, get, put,
    web::{Data, Json, Path},
};
use opentalk_controller_service_facade::OpenTalkControllerService;
use opentalk_types_api_v1::{
    error::ApiError,
    rooms::by_room_id::sip::{PutSipConfigRequestBody, SipConfigResource},
};
use opentalk_types_common::rooms::RoomId;

use crate::api::{
    responses::{Forbidden, InternalServerError, NotFound, Unauthorized},
    v1::response::NoContent,
};

/// Get the sip config for the specified room.
///
/// Returns the sip config if available for the room, otherwise `404 NOT_FOUND`
/// is returned.
#[utoipa::path(
    operation_id = "get_room_sip",
    params(
        ("room_id" = RoomId, description = "The id of the room"),
    ),
    responses(
        (
            status = StatusCode::OK,
            description = "The SIP config is successfully returned",
            body = SipConfigResource,
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
#[get("/rooms/{room_id}/sip")]
pub async fn get(
    service: Data<OpenTalkControllerService>,
    room_id: Path<RoomId>,
) -> Result<Json<SipConfigResource>, ApiError> {
    Ok(Json(service.get_sip_config(room_id.into_inner()).await?))
}

/// Modify the sip configuration of a room. A new sip configuration is created
/// if none was set before.
///
/// Returns the new modified sip configuration.
#[utoipa::path(
    params(
        ("room_id" = RoomId, description = "The id of the room"),
    ),
    request_body = PutSipConfigRequestBody,
    responses(
        (
            status = StatusCode::OK,
            description = "The SIP configuration was updated",
            body = SipConfigResource,
        ),
        (
            status = StatusCode::CREATED,
            description = "A new SIP configuration was created",
            body = SipConfigResource,
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
#[put("/rooms/{room_id}/sip")]
pub async fn put(
    service: Data<OpenTalkControllerService>,
    room_id: Path<RoomId>,
    modify_sip_config: Json<PutSipConfigRequestBody>,
) -> Result<HttpResponse, ApiError> {
    let (sip_config_resource, newly_created) = service
        .set_sip_config(room_id.into_inner(), modify_sip_config.into_inner())
        .await?;

    let mut response = if newly_created {
        HttpResponse::Created()
    } else {
        HttpResponse::Ok()
    };

    Ok(response.json(sip_config_resource))
}

/// Delete the SIP configuration of a room.
///
/// This removes the dial-in functionality from the room.
#[utoipa::path(
    operation_id = "delete_room_sip",
    params(
        ("room_id" = RoomId, description = "The id of the room"),
    ),
    responses(
        (
            status = StatusCode::NO_CONTENT,
            description = "The SIP configuration was successfully deleted",
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
#[delete("/rooms/{room_id}/sip")]
pub async fn delete(
    service: Data<OpenTalkControllerService>,
    room_id: Path<RoomId>,
) -> Result<NoContent, ApiError> {
    service.delete_sip_config(room_id.into_inner()).await?;

    Ok(NoContent)
}
