// SPDX-FileCopyrightText: OpenTalk GmbH <mail@opentalk.eu>
//
// SPDX-License-Identifier: EUPL-1.2

use actix_web::{
    Either, delete, get, patch, post,
    web::{Data, Json, Path, Query, ReqData},
};
use opentalk_controller_service_facade::{OpenTalkControllerService, RequestUser};
use opentalk_types_api_v1::{
    error::ApiError,
    events::{
        DeleteEmailInviteBody, DeleteEventInvitePath, EventInvitee, EventOptionsQuery,
        EventResource, GetEventInstanceResponseBody, PatchEmailInviteBody, PatchInviteBody,
        PostEventInviteBody, PostEventInviteQuery, by_event_id::invites::GetEventsInvitesQuery,
    },
    users::GetEventInvitesPendingResponseBody,
};
use opentalk_types_common::{events::EventId, users::UserId};
use serde::Deserialize;

use super::{ApiResponse, DefaultApiResult};
use crate::api::{
    headers::CursorLink,
    responses::{BadRequest, Forbidden, InternalServerError, NotFound, Unauthorized},
    v1::response::{Created, NoContent},
};

/// Get the invites for an event
///
/// Returns the list of event invites
#[utoipa::path(
    params(
        GetEventsInvitesQuery,
        ("event_id" = EventId, description = "The id of the event"),
    ),
    responses(
        (
            status = StatusCode::OK,
            description = "Event instance successfully returned",
            body = GetEventInstanceResponseBody,
            headers(
                (
                    "link" = CursorLink,
                    description = "Links for paging through the results"
                ),
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
#[get("/events/{event_id}/invites")]
pub async fn get_invites_for_event(
    service: Data<OpenTalkControllerService>,
    current_user: ReqData<RequestUser>,
    event_id: Path<EventId>,
    query: Query<GetEventsInvitesQuery>,
) -> DefaultApiResult<Vec<EventInvitee>> {
    let (invitees, per_page, page, total) = service
        .get_invites_for_event(
            current_user.into_inner(),
            event_id.into_inner(),
            query.into_inner(),
        )
        .await?;

    Ok(ApiResponse::new(invitees).with_page_pagination(per_page, page, total))
}

/// Create a new invite to an event
///
/// Create a new invite to an event with the fields sent in the body.
#[utoipa::path(
    params(
        PostEventInviteQuery,
        ("event_id" = EventId, description = "The id of the event"),
    ),
    request_body = PostEventInviteBody,
    responses(
        (
            status = StatusCode::CREATED,
            description = "The user or email has been invited to the event",
            body = Vec<EventResource>,
        ),
        (
            status = StatusCode::NO_CONTENT,
            description = "The user or email was already invited before, or the user is the creator of the event, in which case they have been invited implicitly",
        ),
        (
            status = StatusCode::BAD_REQUEST,
            response = BadRequest,
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
            status = StatusCode::INTERNAL_SERVER_ERROR,
            response = InternalServerError,
        ),
    ),
    security(
        ("BearerAuth" = []),
    ),
)]
#[post("/events/{event_id}/invites")]
pub async fn create_invite_to_event(
    service: Data<OpenTalkControllerService>,
    current_user: ReqData<RequestUser>,
    event_id: Path<EventId>,
    query: Query<PostEventInviteQuery>,
    create_invite: Json<PostEventInviteBody>,
) -> Result<Either<Created, NoContent>, ApiError> {
    let created = service
        .create_invite_to_event(
            current_user.into_inner(),
            event_id.into_inner(),
            query.into_inner(),
            create_invite.into_inner(),
        )
        .await?;

    match created {
        true => Ok(Either::Left(Created)),
        false => Ok(Either::Right(NoContent)),
    }
}

/// Patch an event invite with the provided fields
///
/// Fields that are not provided in the request body will remain unchanged.
#[utoipa::path(
    request_body = PatchInviteBody,
    params(
        ("event_id" = EventId, description = "The id of the event to be modified"),
        ("user_id" = UserId, description = "The id of the invited user to be modified"),
    ),
    responses(
        (
            status = StatusCode::NO_CONTENT,
            description = "Invite was successfully updated",
        ),
        (
            status = StatusCode::UNAUTHORIZED,
            response = Unauthorized,
        ),
        (
            status = StatusCode::FORBIDDEN,
            description = r"The requesting user does not have the required permissions to update the invite.
              Only the creator of an event can update the invites.",
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
#[patch("/events/{event_id}/invites/{user_id}")]
pub async fn update_invite_to_event(
    service: Data<OpenTalkControllerService>,
    current_user: ReqData<RequestUser>,
    path_parameters: Path<(EventId, UserId)>,
    update_invite: Json<PatchInviteBody>,
) -> Result<NoContent, ApiError> {
    service
        .update_invite_to_event(
            &current_user,
            path_parameters.0,
            path_parameters.1,
            &update_invite,
        )
        .await?;

    Ok(NoContent)
}

/// Patch an event email invite with the provided fields
///
/// Fields that are not provided in the request body will remain unchanged.
#[utoipa::path(
    request_body = PatchEmailInviteBody,
    params(
        ("event_id" = EventId, description = "The id of the event to be modified"),
    ),
    responses(
        (
            status = StatusCode::NO_CONTENT,
            description = "Invite was successfully updated",
        ),
        (
            status = StatusCode::UNAUTHORIZED,
            response = Unauthorized,
        ),
        (
            status = StatusCode::FORBIDDEN,
            description = r"The requesting user does not have the required permissions to update the invite.
              Only the creator of an event can update the invites.",
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
#[patch("/events/{event_id}/invites/email")]
pub async fn update_email_invite_to_event(
    service: Data<OpenTalkControllerService>,
    current_user: ReqData<RequestUser>,
    path_parameters: Path<EventId>,
    update_invite: Json<PatchEmailInviteBody>,
) -> Result<NoContent, ApiError> {
    service
        .update_email_invite_to_event(&current_user, path_parameters.into_inner(), &update_invite)
        .await?;

    Ok(NoContent)
}

/// Query parameters for the `DELETE /events/{event_id}/invites/{user_id}` endpoint
#[derive(Deserialize, Debug, PartialEq, Eq)]
pub struct DeleteEventInviteQuery {
    /// Flag to suppress email notification
    #[serde(default)]
    suppress_email_notification: bool,
}

/// Delete an invite from an event
///
/// This will uninvite the user from the event
#[utoipa::path(
    params(
        DeleteEventInvitePath,
        EventOptionsQuery,
    ),
    responses(
        (
            status = StatusCode::NO_CONTENT,
            description = "The user event invitation has been deleted",
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
#[delete("/events/{event_id}/invites/{user_id}")]
pub async fn delete_invite_to_event(
    service: Data<OpenTalkControllerService>,
    current_user: ReqData<RequestUser>,
    path_params: Path<DeleteEventInvitePath>,
    query: Query<EventOptionsQuery>,
) -> Result<NoContent, ApiError> {
    service
        .delete_invite_to_event(
            current_user.into_inner(),
            path_params.into_inner(),
            query.into_inner(),
        )
        .await?;

    Ok(NoContent)
}

/// Delete an invite from an event
///
/// Delete/Withdraw an event invitation using the email address as the identifier.
///
/// This will also withdraw invites from registered users if the provided email address matches theirs.
#[utoipa::path(
    request_body = DeleteEmailInviteBody,
    params(
        ("event_id" = EventId, description = "The id of the event"),
        EventOptionsQuery,
    ),
    responses(
        (
            status = StatusCode::NO_CONTENT,
            description = "The email event invitation has been deleted",
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
#[delete("/events/{event_id}/invites/email")]
pub async fn delete_email_invite_to_event(
    service: Data<OpenTalkControllerService>,
    current_user: ReqData<RequestUser>,
    path: Path<EventId>,
    query: Query<EventOptionsQuery>,
    body: Json<DeleteEmailInviteBody>,
) -> Result<NoContent, ApiError> {
    service
        .delete_email_invite_to_event(
            current_user.into_inner(),
            path.into_inner(),
            body.into_inner().email,
            query.into_inner(),
        )
        .await?;

    Ok(NoContent)
}

/// Get information about pending invites
///
/// Returns information about pending invites for the current user
#[utoipa::path(
    responses(
        (
            status = StatusCode::OK,
            description = "Information about pending invites is returned",
            body = GetEventInvitesPendingResponseBody,
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
#[get("/users/me/pending_invites")]
pub async fn get_event_invites_pending(
    service: Data<OpenTalkControllerService>,
    current_user: ReqData<RequestUser>,
) -> DefaultApiResult<GetEventInvitesPendingResponseBody> {
    let response = service.get_event_invites_pending(current_user.id).await?;

    Ok(ApiResponse::new(response))
}

/// Accept an invite to an event
///
/// No content required, the request will accept the invitation.
#[utoipa::path(
    params(
        ("event_id" = EventId, description = "The id of the event"),
    ),
    responses(
        (
            status = StatusCode::NO_CONTENT,
            description = "Invitation was accepted",
        ),
        (
            status = StatusCode::NOT_FOUND,
            response = NotFound,
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
            status = StatusCode::INTERNAL_SERVER_ERROR,
            response = InternalServerError,
        ),
    ),
    security(
        ("BearerAuth" = []),
    ),
)]
#[patch("/events/{event_id}/invite")]
pub async fn accept_event_invite(
    service: Data<OpenTalkControllerService>,
    current_user: ReqData<RequestUser>,
    event_id: Path<EventId>,
) -> Result<NoContent, ApiError> {
    service
        .accept_event_invite(current_user.id, event_id.into_inner())
        .await?;

    Ok(NoContent)
}

/// Decline an invite to an event
///
/// No content required, the request will accept the invitation.
#[utoipa::path(
    params(
        ("event_id" = EventId, description = "The id of the event"),
    ),
    responses(
        (
            status = StatusCode::NO_CONTENT,
            description = "Invitation was declined",
        ),
        (
            status = StatusCode::NOT_FOUND,
            response = NotFound,
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
            status = StatusCode::INTERNAL_SERVER_ERROR,
            response = InternalServerError,
        ),
    ),
    security(
        ("BearerAuth" = []),
    ),
)]
#[delete("/events/{event_id}/invite")]
pub async fn decline_event_invite(
    service: Data<OpenTalkControllerService>,
    current_user: ReqData<RequestUser>,
    event_id: Path<EventId>,
) -> Result<NoContent, ApiError> {
    service
        .decline_event_invite(current_user.id, event_id.into_inner())
        .await?;

    Ok(NoContent)
}
