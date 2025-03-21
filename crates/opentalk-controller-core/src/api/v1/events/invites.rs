// SPDX-FileCopyrightText: OpenTalk GmbH <mail@opentalk.eu>
//
// SPDX-License-Identifier: EUPL-1.2

use actix_web::{
    delete, get, patch, post,
    web::{Data, Json, Path, Query, ReqData},
    Either,
};
use kustos::Authz;
use opentalk_controller_service::{
    controller_backend::events::invites::{
        accept_event_invite_inner, create_invite_to_event_inner, decline_event_invite_inner,
        delete_email_invite_to_event_inner, delete_invite_to_event_inner,
        get_event_invites_pending_inner, get_invites_for_event_inner,
        update_email_invite_to_event_inner, update_invite_to_event_inner,
    },
    services::MailService,
};
use opentalk_database::Db;
use opentalk_db_storage::{tenants::Tenant, users::User};
use opentalk_keycloak_admin::KeycloakAdminClient;
use opentalk_types_api_v1::{
    error::ApiError,
    events::{
        by_event_id::invites::GetEventsInvitesQuery, DeleteEmailInviteBody, DeleteEventInvitePath,
        EventInvitee, EventOptionsQuery, EventResource, GetEventInstanceResponseBody,
        PatchEmailInviteBody, PatchInviteBody, PostEventInviteBody, PostEventInviteQuery,
    },
    users::GetEventInvitesPendingResponseBody,
};
use opentalk_types_common::{events::EventId, users::UserId};
use serde::Deserialize;

use super::{ApiResponse, DefaultApiResult};
use crate::{
    api::{
        headers::CursorLink,
        responses::{BadRequest, Forbidden, InternalServerError, NotFound, Unauthorized},
        v1::response::{Created, NoContent},
    },
    settings::SharedSettingsActix,
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
    settings: SharedSettingsActix,
    db: Data<Db>,
    kc_admin_client: Data<KeycloakAdminClient>,
    current_tenant: ReqData<Tenant>,
    event_id: Path<EventId>,
    query: Query<GetEventsInvitesQuery>,
) -> DefaultApiResult<Vec<EventInvitee>> {
    let (invitees, per_page, page, total) = get_invites_for_event_inner(
        &settings.load_full(),
        &db,
        &kc_admin_client,
        &current_tenant,
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
#[allow(clippy::too_many_arguments)]
pub async fn create_invite_to_event(
    settings: SharedSettingsActix,
    db: Data<Db>,
    authz: Data<Authz>,
    kc_admin_client: Data<KeycloakAdminClient>,
    current_tenant: ReqData<Tenant>,
    current_user: ReqData<User>,
    event_id: Path<EventId>,
    query: Query<PostEventInviteQuery>,
    create_invite: Json<PostEventInviteBody>,
    mail_service: Data<MailService>,
) -> Result<Either<Created, NoContent>, ApiError> {
    let created = create_invite_to_event_inner(
        &settings.load_full(),
        &db,
        &authz,
        &kc_admin_client,
        &current_tenant,
        current_user.into_inner(),
        event_id.into_inner(),
        query.into_inner(),
        create_invite.into_inner(),
        &mail_service,
    )
    .await?;

    Ok(if created {
        Either::Left(Created)
    } else {
        Either::Right(NoContent)
    })
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
    db: Data<Db>,
    current_user: ReqData<User>,
    path_parameters: Path<(EventId, UserId)>,
    update_invite: Json<PatchInviteBody>,
) -> Result<NoContent, ApiError> {
    update_invite_to_event_inner(
        &db,
        &current_user,
        path_parameters.into_inner(),
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
    db: Data<Db>,
    current_user: ReqData<User>,
    path_parameters: Path<EventId>,
    update_invite: Json<PatchEmailInviteBody>,
) -> Result<NoContent, ApiError> {
    update_email_invite_to_event_inner(
        &db,
        &current_user,
        path_parameters.into_inner(),
        &update_invite,
    )
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
#[allow(clippy::too_many_arguments)]
pub async fn delete_invite_to_event(
    settings: SharedSettingsActix,
    db: Data<Db>,
    kc_admin_client: Data<KeycloakAdminClient>,
    current_tenant: ReqData<Tenant>,
    current_user: ReqData<User>,
    authz: Data<Authz>,
    path_params: Path<DeleteEventInvitePath>,
    query: Query<EventOptionsQuery>,
    mail_service: Data<MailService>,
) -> Result<NoContent, ApiError> {
    delete_invite_to_event_inner(
        &settings.load_full(),
        &db,
        &kc_admin_client,
        current_tenant.into_inner(),
        current_user.into_inner(),
        &authz,
        path_params.into_inner(),
        query.into_inner(),
        &mail_service,
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
#[allow(clippy::too_many_arguments)]
pub async fn delete_email_invite_to_event(
    settings: SharedSettingsActix,
    db: Data<Db>,
    kc_admin_client: Data<KeycloakAdminClient>,
    current_tenant: ReqData<Tenant>,
    current_user: ReqData<User>,
    authz: Data<Authz>,
    path: Path<EventId>,
    query: Query<EventOptionsQuery>,
    mail_service: Data<MailService>,
    body: Json<DeleteEmailInviteBody>,
) -> Result<NoContent, ApiError> {
    delete_email_invite_to_event_inner(
        &settings.load_full(),
        &db,
        &kc_admin_client,
        current_tenant.into_inner(),
        current_user.into_inner(),
        &authz,
        path.into_inner(),
        query.into_inner(),
        &mail_service,
        body.into_inner().email,
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
    db: Data<Db>,
    current_user: ReqData<User>,
) -> DefaultApiResult<GetEventInvitesPendingResponseBody> {
    let response = get_event_invites_pending_inner(&db, current_user.id).await?;

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
    db: Data<Db>,
    current_user: ReqData<User>,
    event_id: Path<EventId>,
) -> Result<NoContent, ApiError> {
    accept_event_invite_inner(&db, current_user.into_inner().id, event_id.into_inner()).await?;

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
    db: Data<Db>,
    current_user: ReqData<User>,
    event_id: Path<EventId>,
) -> Result<NoContent, ApiError> {
    decline_event_invite_inner(&db, current_user.id, event_id.into_inner()).await?;

    Ok(NoContent)
}
