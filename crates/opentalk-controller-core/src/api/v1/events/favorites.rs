// SPDX-FileCopyrightText: OpenTalk GmbH <mail@opentalk.eu>
//
// SPDX-License-Identifier: EUPL-1.2

use actix_web::{
    Either, delete, put,
    web::{Data, Path, ReqData},
};
use opentalk_controller_service_facade::{OpenTalkControllerService, RequestUser};
use opentalk_types_api_v1::error::ApiError;
use opentalk_types_common::events::EventId;

use crate::api::{
    responses::{InternalServerError, NotFound, Unauthorized},
    v1::response::{Created, NoContent},
};

/// Add an event to the current user's favorites
///
/// The event will be marked as favorited by the calling user.
#[utoipa::path(
    params(
        ("event_id" = EventId, description = "The id of the event that gets marked as favorite"),
    ),
    responses(
        (
            status = StatusCode::CREATED,
            description = "The event has been addded to the user's favorites",
        ),
        (
            status = StatusCode::NO_CONTENT,
            description = "The event had already been added to the user's favorites, no changes made",
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
#[put("/users/me/event_favorites/{event_id}")]
pub async fn add_event_to_favorites(
    service: Data<OpenTalkControllerService>,
    current_user: ReqData<RequestUser>,
    event_id: Path<EventId>,
) -> Result<Either<Created, NoContent>, ApiError> {
    let created = service
        .add_event_to_favorites(current_user.into_inner(), event_id.into_inner())
        .await?;

    match created {
        true => Ok(Either::Left(Created)),
        false => Ok(Either::Right(NoContent)),
    }
}

/// Remove an event from the current user's favorites
///
/// The event will be marked as non-favorited by the calling user.
#[utoipa::path(
    params(
        ("event_id" = EventId, description = "The id of the event that gets marked as non-favorited"),
    ),
    responses(
        (
            status = StatusCode::NO_CONTENT,
            description = "The event has been removed from the user's favorites",
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
#[delete("/users/me/event_favorites/{event_id}")]
pub async fn remove_event_from_favorites(
    service: Data<OpenTalkControllerService>,
    current_user: ReqData<RequestUser>,
    event_id: Path<EventId>,
) -> Result<NoContent, ApiError> {
    service
        .remove_event_from_favorites(current_user.into_inner(), event_id.into_inner())
        .await?;

    Ok(NoContent)
}
