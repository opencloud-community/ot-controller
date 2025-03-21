// SPDX-FileCopyrightText: OpenTalk GmbH <mail@opentalk.eu>
//
// SPDX-License-Identifier: EUPL-1.2

use actix_web::{
    delete, get, patch, post,
    web::{Data, Json, Path, Query, ReqData},
    Either,
};
use chrono::{DateTime, Utc};
use kustos::{
    policies_builder::{GrantingAccess, PoliciesBuilder},
    prelude::{AccessMethod, IsSubject},
    Authz, Resource,
};
use opentalk_controller_service::{
    controller_backend::events::{
        delete_event_inner, get_event_inner, get_events_inner, new_event_inner, patch_event_inner,
    },
    services::MailService,
};
use opentalk_database::Db;
use opentalk_db_storage::{tenants::Tenant, users::User};
use opentalk_keycloak_admin::KeycloakAdminClient;
use opentalk_signaling_core::{ExchangeHandle, ObjectStorage};
use opentalk_types_api_v1::{
    error::ApiError,
    events::{
        DeleteEventsQuery, EventOptionsQuery, EventOrException, EventResource, GetEventQuery,
        GetEventsQuery, PatchEventBody, PatchEventQuery, PostEventsBody,
    },
};
use opentalk_types_common::{events::EventId, time::RecurrencePattern};
use serde::Deserialize;

use super::{response::NoContent, ApiResponse, DefaultApiResult};
use crate::{
    api::{
        headers::CursorLink,
        responses::{BadRequest, Forbidden, InternalServerError, NotFound, Unauthorized},
    },
    settings::SharedSettingsActix,
};

pub mod favorites;
pub mod instances;
pub mod invites;
pub mod shared_folder;

/// Create a new event
///
/// Create a new event with the fields sent in the body.
#[utoipa::path(
    params(EventOptionsQuery),
    responses(
        (
            status = StatusCode::CREATED,
            description = "The event has been created",
            body = EventResource,
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
            status = StatusCode::INTERNAL_SERVER_ERROR,
            response = InternalServerError,
        ),
    ),
    security(
        ("BearerAuth" = []),
    ),
)]
#[post("/events")]
pub async fn new_event(
    settings: SharedSettingsActix,
    db: Data<Db>,
    authz: Data<Authz>,
    current_user: ReqData<User>,
    new_event: Json<PostEventsBody>,
    query: Query<EventOptionsQuery>,
    mail_service: Data<MailService>,
) -> DefaultApiResult<EventResource> {
    let event_resource = new_event_inner(
        &settings.load_full(),
        &db,
        &authz,
        current_user.into_inner(),
        new_event.into_inner(),
        query.into_inner(),
        &mail_service,
    )
    .await?;

    Ok(ApiResponse::new(event_resource))
}

/// Get a list of events accessible by the requesting user
///
/// Returns a paginated list of events and their exceptions inside the given time range
#[utoipa::path(
    params(GetEventsQuery),
    responses(
        (
            status = StatusCode::OK,
            description = "List of the events and exceptions",
            body = Vec<EventOrException>,
            headers(
                (
                    "link" = CursorLink,
                    description = "Links for paging through the results"
                ),
            ),
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
            status = StatusCode::INTERNAL_SERVER_ERROR,
            response = InternalServerError,
        ),
    ),
    security(
        ("BearerAuth" = []),
    ),
)]
#[get("/events")]
pub async fn get_events(
    settings: SharedSettingsActix,
    db: Data<Db>,
    kc_admin_client: Data<KeycloakAdminClient>,
    current_tenant: ReqData<Tenant>,
    current_user: ReqData<User>,
    query: Query<GetEventsQuery>,
) -> DefaultApiResult<Vec<EventOrException>> {
    let (event_resources, before, after) = get_events_inner(
        &settings.load_full(),
        &db,
        &kc_admin_client,
        current_tenant.into_inner(),
        current_user.into_inner(),
        query.into_inner(),
    )
    .await?;

    Ok(ApiResponse::new(event_resources).with_cursor_pagination(before, after))
}

/// Get an event
///
/// Returns the event resource for the given id
#[utoipa::path(
    params(
        GetEventQuery,
        ("event_id" = EventId, description = "The id of the event"),
    ),
    responses(
        (
            status = StatusCode::OK,
            description = "Event was successfully retrieved",
            body = EventResource
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
#[get("/events/{event_id}")]
pub async fn get_event(
    settings: SharedSettingsActix,
    db: Data<Db>,
    kc_admin_client: Data<KeycloakAdminClient>,
    current_tenant: ReqData<Tenant>,
    current_user: ReqData<User>,
    event_id: Path<EventId>,
    query: Query<GetEventQuery>,
) -> DefaultApiResult<EventResource> {
    let event_resource = get_event_inner(
        &settings.load_full(),
        &db,
        &kc_admin_client,
        current_tenant.into_inner(),
        current_user.into_inner(),
        event_id.into_inner(),
        query.into_inner(),
    )
    .await?;

    Ok(ApiResponse::new(event_resource))
}

/// Patch an event
///
/// Fields that are not provided in the request body will remain unchanged.
#[allow(clippy::too_many_arguments)]
#[utoipa::path(
    request_body = PatchEventBody,
    params(
        PatchEventQuery,
        ("event_id" = EventId, description = "The id of the event"),
    ),
    responses(
        (
            status = StatusCode::OK,
            description = "The event was successfully updated",
            body = EventResource
        ),
        (
            status = StatusCode::NO_CONTENT,
            description = "The patch was empty",
        ),
        (
            status = StatusCode::BAD_REQUEST,
            description = r"Could not modify the specified event due to wrong
                syntax or bad values, for example an invalid timestamp string",
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
#[patch("/events/{event_id}")]
pub async fn patch_event(
    settings: SharedSettingsActix,
    db: Data<Db>,
    authz: Data<Authz>,
    kc_admin_client: Data<KeycloakAdminClient>,
    current_tenant: ReqData<Tenant>,
    current_user: ReqData<User>,
    event_id: Path<EventId>,
    query: Query<PatchEventQuery>,
    patch: Json<PatchEventBody>,
    mail_service: Data<MailService>,
) -> Result<Either<ApiResponse<EventResource>, NoContent>, ApiError> {
    let response = patch_event_inner(
        &settings.load_full(),
        &db,
        &authz,
        &kc_admin_client,
        current_tenant.into_inner(),
        current_user.into_inner(),
        event_id.into_inner(),
        query.into_inner(),
        patch.into_inner(),
        &mail_service,
    )
    .await?;

    Ok(match response {
        futures::future::Either::Left(event_resource) => {
            Either::Left(ApiResponse::new(event_resource))
        }
        futures::future::Either::Right(()) => Either::Right(NoContent),
    })
}

/// Delete an event and its owned resources, including the associated room.
///
/// Deletes the event by the id if found. See the query parameters for affecting
/// the behavior of this endpoint, such as mail notification suppression, or
/// succeding even if external resources cannot be successfully deleted.
#[utoipa::path(
    params(
        DeleteEventsQuery,
        ("event_id" = EventId, description = "The id of the event"),
    ),
    responses(
        (
            status = StatusCode::NO_CONTENT,
            description = "The event was successfully deleted",
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
#[delete("/events/{event_id}")]
#[allow(clippy::too_many_arguments)]
pub async fn delete_event(
    settings: SharedSettingsActix,
    db: Data<Db>,
    storage: Data<ObjectStorage>,
    exchange_handle: Data<ExchangeHandle>,
    kc_admin_client: Data<KeycloakAdminClient>,
    current_tenant: ReqData<Tenant>,
    current_user: ReqData<User>,
    authz: Data<Authz>,
    query: Query<DeleteEventsQuery>,
    event_id: Path<EventId>,
    mail_service: Data<MailService>,
) -> Result<NoContent, ApiError> {
    delete_event_inner(
        &settings.load_full(),
        &db,
        &storage,
        &exchange_handle,
        &kc_admin_client,
        current_tenant.into_inner(),
        current_user.into_inner(),
        &authz,
        query.into_inner(),
        event_id.into_inner(),
        &mail_service,
    )
    .await?;

    Ok(NoContent)
}

#[derive(Deserialize)]
pub struct EventRescheduleBody {
    _from: DateTime<Utc>,
    _is_all_day: Option<bool>,
    _starts_at: Option<bool>,
    _ends_at: Option<bool>,
    _recurrence_pattern: RecurrencePattern,
}

#[post("/events/{event_id}/reschedule")]
pub async fn event_reschedule(
    _db: Data<Db>,
    _event_id: Path<EventId>,
    _body: Json<EventRescheduleBody>,
) -> actix_web::HttpResponse {
    actix_web::HttpResponse::NotImplemented().finish()
}

/// Helper trait to to reduce boilerplate in the single route handlers
///
/// Bundles multiple resources into groups.
pub trait EventPoliciesBuilderExt {
    fn event_read_access(self, event_id: EventId) -> Self;
    fn event_write_access(self, event_id: EventId) -> Self;

    fn event_invite_invitee_access(self, event_id: EventId) -> Self;
}

impl<T> EventPoliciesBuilderExt for PoliciesBuilder<GrantingAccess<T>>
where
    T: IsSubject + Clone,
{
    /// GET access to the event and related endpoints.
    /// PUT and DELETE to the event_favorites endpoint.
    fn event_read_access(self, event_id: EventId) -> Self {
        self.add_resource(event_id.resource_id(), [AccessMethod::Get])
            .add_resource(
                event_id.resource_id().with_suffix("/instances"),
                [AccessMethod::Get],
            )
            .add_resource(
                event_id.resource_id().with_suffix("/instances/*"),
                [AccessMethod::Get],
            )
            .add_resource(
                event_id.resource_id().with_suffix("/invites"),
                [AccessMethod::Get],
            )
            .add_resource(
                event_id.resource_id().with_suffix("/shared_folder"),
                [AccessMethod::Get],
            )
            .add_resource(
                format!("/users/me/event_favorites/{event_id}"),
                [AccessMethod::Put, AccessMethod::Delete],
            )
    }

    /// PATCH and DELETE to the event
    /// POST to reschedule and invites of the event
    /// PATCH to instances
    /// DELETE to invites
    fn event_write_access(self, event_id: EventId) -> Self {
        self.add_resource(
            event_id.resource_id(),
            [AccessMethod::Patch, AccessMethod::Delete],
        )
        .add_resource(
            event_id.resource_id().with_suffix("/reschedule"),
            [AccessMethod::Post],
        )
        .add_resource(
            event_id.resource_id().with_suffix("/instances/*"),
            [AccessMethod::Patch],
        )
        .add_resource(
            event_id.resource_id().with_suffix("/invites"),
            [AccessMethod::Post],
        )
        .add_resource(
            event_id.resource_id().with_suffix("/invites/*"),
            [AccessMethod::Patch, AccessMethod::Delete],
        )
        .add_resource(
            event_id.resource_id().with_suffix("/shared_folder"),
            [AccessMethod::Put, AccessMethod::Delete],
        )
    }

    /// PATCH and DELETE to event invite
    fn event_invite_invitee_access(self, event_id: EventId) -> Self {
        self.add_resource(
            format!("/events/{event_id}/invite"),
            [AccessMethod::Patch, AccessMethod::Delete],
        )
    }
}
