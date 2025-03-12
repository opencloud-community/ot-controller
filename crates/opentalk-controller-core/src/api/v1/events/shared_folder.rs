// SPDX-FileCopyrightText: OpenTalk GmbH <mail@opentalk.eu>
//
// SPDX-License-Identifier: EUPL-1.2

use actix_http::StatusCode;
use actix_web::{
    delete, get, put,
    web::{Data, Json, Path, Query, ReqData},
    CustomizeResponder, Responder as _,
};
use opentalk_controller_service_facade::{OpenTalkControllerService, RequestUser};
use opentalk_types_api_v1::{
    error::ApiError,
    events::{DeleteSharedFolderQuery, PutSharedFolderQuery},
};
use opentalk_types_common::{events::EventId, shared_folders::SharedFolder};

use crate::api::{
    responses::{Forbidden, InternalServerError, NotFound, Unauthorized},
    v1::response::NoContent,
};

/// Get the shared folder for an event
///
/// Returns the shared folder for an event if available
#[utoipa::path(
    params(
        ("event_id" = EventId, description = "The id of the event"),
    ),
    responses(
        (
            status = StatusCode::OK,
            description = "Shared folder returned",
            body = SharedFolder,
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
#[get("/events/{event_id}/shared_folder")]
pub async fn get_shared_folder_for_event(
    service: Data<OpenTalkControllerService>,
    current_user: ReqData<RequestUser>,
    event_id: Path<EventId>,
) -> Result<Json<SharedFolder>, ApiError> {
    let shared_folder = service
        .get_shared_folder_for_event(current_user.into_inner(), event_id.into_inner())
        .await?;

    Ok(Json(shared_folder))
}

/// Create a shared folder for an event
///
/// Returns the shared folder for an event if created
#[utoipa::path(
    params(
        PutSharedFolderQuery,
        ("event_id" = EventId, description = "The id of the event"),
    ),
    responses(
        (
            status = StatusCode::OK,
            description = "Shared folder created",
            body = SharedFolder,
        ),
        (
            status = StatusCode::NOT_MODIFIED,
            description = "Shared folder was already present",
            body = SharedFolder,
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
#[put("/events/{event_id}/shared_folder")]
pub async fn put_shared_folder_for_event(
    service: Data<OpenTalkControllerService>,
    current_user: ReqData<RequestUser>,
    event_id: Path<EventId>,
    query: Query<PutSharedFolderQuery>,
) -> Result<CustomizeResponder<Json<SharedFolder>>, ApiError> {
    let (shared_folder, created) = service
        .put_shared_folder_for_event(
            current_user.into_inner(),
            event_id.into_inner(),
            query.into_inner(),
        )
        .await?;

    Ok(Json(shared_folder).customize().with_status(if created {
        StatusCode::CREATED
    } else {
        StatusCode::OK
    }))
}

/// Delete the shared folder of an event
///
/// Will delete the shared folder from the external system and remove the reference to it
#[utoipa::path(
    params(
        ("event_id" = EventId, description = "The id of the event"),
        DeleteSharedFolderQuery,
    ),
    responses(
        (
            status = StatusCode::NO_CONTENT,
            description = "Shared folder was successfully deleted, or no shared folder had been present",
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
#[delete("/events/{event_id}/shared_folder")]
pub async fn delete_shared_folder_for_event(
    service: Data<OpenTalkControllerService>,
    current_user: ReqData<RequestUser>,
    event_id: Path<EventId>,
    query: Query<DeleteSharedFolderQuery>,
) -> Result<NoContent, ApiError> {
    service
        .delete_shared_folder_for_event(
            current_user.into_inner(),
            event_id.into_inner(),
            query.into_inner(),
        )
        .await?;

    Ok(NoContent)
}
