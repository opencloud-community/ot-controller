// SPDX-FileCopyrightText: OpenTalk GmbH <mail@opentalk.eu>
//
// SPDX-License-Identifier: EUPL-1.2

use actix_web::{
    dev::HttpServiceFactory,
    error::Result,
    post,
    web::{Data, Json},
};
use opentalk_controller_service_facade::OpenTalkControllerService;
use opentalk_types_api_v1::{
    error::{ApiError, ErrorBody},
    services::{PostServiceStartResponseBody, call_in::PostCallInStartRequestBody},
};

use crate::api::responses::{InternalServerError, Unauthorized};

// Note to devs:
// Please update `docs/admin/keycloak.md` service login documentation as well if
// you change something here
pub const REQUIRED_CALL_IN_ROLE: &str = "opentalk-call-in";

/// Starts a signaling session for call-in
///
/// Takes call-in id and pin and returns a ticket for the `/signaling` endpoint. Behaves similar to the
/// `/rooms/{room_id}/start` endpoint.
///
/// This endpoint is provided for call-in gateways to start a room connection
/// for call-in participants. The participant typically has to provide the
/// credentials (id and pin) via DTMF (the number pad).
#[utoipa::path(
    context_path = "/services/call_in",
    request_body = PostCallInStartRequestBody,
    responses(
        (
            status = StatusCode::OK,
            description = "The dial-in participant has successfully \
                authenticated for the room. Information needed for connecting to the signaling \
                is contained in the response",
            body = PostServiceStartResponseBody,
        ),
        (
            status = StatusCode::UNAUTHORIZED,
            response = Unauthorized,
        ),
        (
            status = StatusCode::BAD_REQUEST,
            description = "`id` and `pin` are not valid for any room.",
            body = ErrorBody,
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
#[post("/start")]
pub async fn post_call_in_start(
    service: Data<OpenTalkControllerService>,
    request: Json<PostCallInStartRequestBody>,
) -> Result<Json<PostServiceStartResponseBody>, ApiError> {
    let response = service.start_call_in(request.into_inner()).await?;

    Ok(Json(response))
}

pub fn services() -> impl HttpServiceFactory {
    actix_web::web::scope("/call_in")
        .wrap(super::RequiredRealmRole::new(REQUIRED_CALL_IN_ROLE))
        .service(post_call_in_start)
}
