// SPDX-FileCopyrightText: OpenTalk GmbH <mail@opentalk.eu>
//
// SPDX-License-Identifier: EUPL-1.2

use crate::api::v1::response::{ApiError, Created, NoContent};
use actix_web::web::{Data, Path, ReqData};
use actix_web::{delete, put, Either};
use database::Db;
use db_storage::events::{Event, EventFavorite, NewEventFavorite};
use db_storage::users::User;
use types::core::EventId;

/// API Endpoint *PUT /users/me/event_favorites/{event_id}*
///
/// Add an event to the current users favorites
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
