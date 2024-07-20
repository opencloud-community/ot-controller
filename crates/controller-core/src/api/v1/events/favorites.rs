// SPDX-FileCopyrightText: OpenTalk GmbH <mail@opentalk.eu>
//
// SPDX-License-Identifier: EUPL-1.2

use actix_web::{
    delete, put,
    web::{Data, Path, ReqData},
    Either,
};
use opentalk_database::Db;
use opentalk_db_storage::{
    events::{Event, EventFavorite, NewEventFavorite},
    users::User,
};
use opentalk_types::{api::error::ApiError, core::EventId};

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
    db: Data<Db>,
    id: Path<EventId>,
    current_user: ReqData<User>,
) -> Result<Either<Created, NoContent>, ApiError> {
    let event_id = id.into_inner();

    let mut conn = db.get_conn().await?;

    let _event = Event::get(&mut conn, event_id).await?;

    let result = NewEventFavorite {
        user_id: current_user.id,
        event_id,
    }
    .try_insert(&mut conn)
    .await?;

    match result {
        Some(_) => Ok(Either::Left(Created)),
        None => Ok(Either::Right(NoContent)),
    }
}

/// API Endpoint *DELETE /users/me/event_favorites/{event_id}*
///
/// Remove an event from the current users favorites
#[delete("/users/me/event_favorites/{event_id}")]
pub async fn remove_event_from_favorites(
    db: Data<Db>,
    id: Path<EventId>,
    current_user: ReqData<User>,
) -> Result<NoContent, ApiError> {
    let event_id = id.into_inner();

    let mut conn = db.get_conn().await?;

    let existed = EventFavorite::delete_by_id(&mut conn, current_user.id, event_id).await?;

    if existed {
        Ok(NoContent)
    } else {
        Err(ApiError::not_found())
    }
}
