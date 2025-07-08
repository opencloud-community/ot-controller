// SPDX-FileCopyrightText: OpenTalk GmbH <mail@opentalk.eu>
//
// SPDX-License-Identifier: EUPL-1.2

use actix_web::{
    Either, get, patch,
    web::{Data, Json, Path, Query, ReqData},
};
use opentalk_controller_service_facade::{OpenTalkControllerService, RequestUser};
use opentalk_types_api_v1::{
    error::ApiError,
    events::{
        EventInstance, EventInstancePath, EventInstanceQuery, GetEventInstanceResponseBody,
        GetEventInstancesQuery, GetEventInstancesResponseBody, PatchEventInstanceBody,
    },
};
use opentalk_types_common::events::EventId;

use super::{ApiResponse, DefaultApiResult};
use crate::api::{
    headers::PageLink,
    responses::{Forbidden, InternalServerError, NotFound, Unauthorized},
    v1::response::NoContent,
};

/// Get a list of the instances of an event
///
/// The instances are calculated based on the RRULE of the event. If no RRULE is
/// set for the event, the single event instance is returned.
///
/// If no pagination query is added, the default page size is used.
#[utoipa::path(
    params(
        GetEventInstancesQuery,
        ("event_id" = EventId, description = "The id of the event")
    ),
    responses(
        (
            status = StatusCode::OK,
            description = "List of event instances successfully returned",
            body = GetEventInstancesResponseBody,
            headers(
                ("link" = PageLink, description = "Links for paging through the results"),
            ),
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
#[get("/events/{event_id}/instances")]
pub async fn get_event_instances(
    service: Data<OpenTalkControllerService>,
    current_user: ReqData<RequestUser>,
    event_id: Path<EventId>,
    query: Query<GetEventInstancesQuery>,
) -> DefaultApiResult<GetEventInstancesResponseBody> {
    let (event_instances, before, after) = service
        .get_event_instances(
            &current_user.into_inner(),
            event_id.into_inner(),
            query.into_inner(),
        )
        .await?;

    Ok(ApiResponse::new(event_instances).with_cursor_pagination(before, after))
}

/// Get an event instance
///
/// Returns the event instance resource
#[utoipa::path(
    params(
        EventInstancePath,
        EventInstanceQuery,
    ),
    responses(
        (
            status = StatusCode::OK,
            description = "Event instance successfully returned",
            body = GetEventInstanceResponseBody,
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
#[get("/events/{event_id}/instances/{instance_id}")]
pub async fn get_event_instance(
    service: Data<OpenTalkControllerService>,
    current_user: ReqData<RequestUser>,
    path: Path<EventInstancePath>,
    query: Query<EventInstanceQuery>,
) -> DefaultApiResult<GetEventInstanceResponseBody> {
    let response = service
        .get_event_instance(
            &current_user.into_inner(),
            path.into_inner(),
            query.into_inner(),
        )
        .await?;

    Ok(ApiResponse::new(response))
}

/// Modifies an event instance
///
/// Patch an instance of a recurring event. This creates or modifies an exception for the event
/// at the point of time of the given instance_id.
/// Returns the patched event instance
#[utoipa::path(
    params(
        EventInstancePath,
        EventInstanceQuery,
    ),
    request_body = PatchEventInstanceBody,
    responses(
        (
            status = StatusCode::OK,
            description = "Event instance successfully updated",
            body = EventInstance,
        ),
        (
            status = StatusCode::NO_CONTENT,
            description = "The request body was empty, no changes required",
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
#[patch("/events/{event_id}/instances/{instance_id}")]
pub async fn patch_event_instance(
    service: Data<OpenTalkControllerService>,
    current_user: ReqData<RequestUser>,
    path: Path<EventInstancePath>,
    query: Query<EventInstanceQuery>,
    patch: Json<PatchEventInstanceBody>,
) -> Result<Either<ApiResponse<EventInstance>, NoContent>, ApiError> {
    let event_instance = service
        .patch_event_instance(
            current_user.into_inner(),
            path.into_inner(),
            query.into_inner(),
            patch.into_inner(),
        )
        .await?;

    match event_instance {
        Some(event_instance) => Ok(Either::Left(ApiResponse::new(event_instance))),
        _ => Ok(Either::Right(NoContent)),
    }
}
