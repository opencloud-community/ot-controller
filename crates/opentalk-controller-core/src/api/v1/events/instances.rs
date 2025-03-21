// SPDX-FileCopyrightText: OpenTalk GmbH <mail@opentalk.eu>
//
// SPDX-License-Identifier: EUPL-1.2

use actix_web::{
    get, patch,
    web::{Data, Json, Path, Query, ReqData},
    Either,
};
use kustos::Authz;
use opentalk_controller_service::{
    controller_backend::events::instances::{
        get_event_instance_inner, get_event_instances_inner, patch_event_instance_inner,
    },
    services::MailService,
};
use opentalk_database::Db;
use opentalk_db_storage::{tenants::Tenant, users::User};
use opentalk_keycloak_admin::KeycloakAdminClient;
use opentalk_types_api_v1::{
    error::ApiError,
    events::{
        EventInstance, EventInstancePath, EventInstanceQuery, GetEventInstanceResponseBody,
        GetEventInstancesQuery, GetEventInstancesResponseBody, PatchEventInstanceBody,
    },
};
use opentalk_types_common::events::EventId;

use super::{ApiResponse, DefaultApiResult};
use crate::{
    api::{
        headers::PageLink,
        responses::{Forbidden, InternalServerError, NotFound, Unauthorized},
        v1::response::NoContent,
    },
    settings::SharedSettingsActix,
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
    settings: SharedSettingsActix,
    db: Data<Db>,
    kc_admin_client: Data<KeycloakAdminClient>,
    current_tenant: ReqData<Tenant>,
    current_user: ReqData<User>,
    event_id: Path<EventId>,
    query: Query<GetEventInstancesQuery>,
) -> DefaultApiResult<GetEventInstancesResponseBody> {
    let (event_instances, before, after) = get_event_instances_inner(
        &settings.load_full(),
        &db,
        &kc_admin_client,
        &current_tenant,
        &current_user,
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
    settings: SharedSettingsActix,
    db: Data<Db>,
    kc_admin_client: Data<KeycloakAdminClient>,
    current_tenant: ReqData<Tenant>,
    current_user: ReqData<User>,
    path: Path<EventInstancePath>,
    query: Query<EventInstanceQuery>,
) -> DefaultApiResult<GetEventInstanceResponseBody> {
    let response = get_event_instance_inner(
        &settings.load_full(),
        &db,
        &kc_admin_client,
        &current_tenant,
        &current_user,
        path.into_inner(),
        query.into_inner(),
    )
    .await?;

    Ok(ApiResponse::new(response))
}

/// API Endpoint `PATCH /events/{event_id}/{instance_id}`
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
#[allow(clippy::too_many_arguments)]
pub async fn patch_event_instance(
    settings: SharedSettingsActix,
    db: Data<Db>,
    authz: Data<Authz>,
    kc_admin_client: Data<KeycloakAdminClient>,
    current_tenant: ReqData<Tenant>,
    current_user: ReqData<User>,
    path: Path<EventInstancePath>,
    query: Query<EventInstanceQuery>,
    patch: Json<PatchEventInstanceBody>,
    mail_service: Data<MailService>,
) -> Result<Either<ApiResponse<EventInstance>, NoContent>, ApiError> {
    let response = patch_event_instance_inner(
        &settings.load_full(),
        &db,
        &authz,
        &kc_admin_client,
        current_tenant.into_inner(),
        current_user.into_inner(),
        path.into_inner(),
        query.into_inner(),
        patch.into_inner(),
        &mail_service,
    )
    .await?;

    Ok(match response {
        futures::future::Either::Left(event_instance) => {
            Either::Left(ApiResponse::new(event_instance))
        }
        futures::future::Either::Right(()) => Either::Right(NoContent),
    })
}
