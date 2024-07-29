// SPDX-FileCopyrightText: OpenTalk GmbH <mail@opentalk.eu>
//
// SPDX-License-Identifier: EUPL-1.2

use std::sync::Arc;

use actix_web::{
    delete, get, patch, post,
    web::{Data, Json, Path, Query, ReqData},
    Either,
};
use chrono::Utc;
use diesel_async::{scoped_futures::ScopedFutureExt, AsyncConnection};
use kustos::{policies_builder::PoliciesBuilder, Authz};
use opentalk_controller_settings::Settings;
use opentalk_database::Db;
use opentalk_db_storage::{
    events::{
        email_invites::{EventEmailInvite, NewEventEmailInvite, UpdateEventEmailInvite},
        shared_folders::EventSharedFolder,
        Event, EventFavorite, EventInvite, NewEventInvite, UpdateEventInvite,
    },
    invites::NewInvite,
    rooms::Room,
    sip_configs::SipConfig,
    streaming_targets::get_room_streaming_targets,
    tenants::Tenant,
    users::User,
};
use opentalk_keycloak_admin::KeycloakAdminClient;
use opentalk_types::{
    api::{
        error::ApiError,
        v1::{
            events::{
                invites::GetEventsInvitesQuery, DeleteEmailInviteBody, DeleteEventInvitePath,
                EmailInvite, EventOptionsQuery, PatchEmailInviteBody, PatchInviteBody,
                PostEventInviteBody, PostEventInviteQuery, UserInvite,
            },
            pagination::PagePaginationQuery,
            users::GetEventInvitesPendingResponse,
        },
    },
    common::{email::EmailAddress, shared_folder::SharedFolder, streaming::RoomStreamingTarget},
    core::{EmailInviteRole, EventId, EventInviteStatus, RoomId, UserId},
};
use serde::Deserialize;
use snafu::Report;

use super::{ApiResponse, DefaultApiResult};
use crate::{
    api::{
        responses::{BadRequest, Forbidden, InternalServerError, NotFound, Unauthorized},
        v1::{
            events::{
                enrich_from_keycloak, enrich_invitees_from_keycloak,
                get_invited_mail_recipients_for_event, get_tenant_filter, EventInvitee,
                EventInviteeExt, EventPoliciesBuilderExt,
            },
            response::{Created, NoContent},
            rooms::RoomsPoliciesBuilderExt,
        },
    },
    services::{
        ExternalMailRecipient, MailRecipient, MailService, RegisteredMailRecipient,
        UnregisteredMailRecipient,
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
    let settings = settings.load_full();
    let event_id = event_id.into_inner();
    let GetEventsInvitesQuery {
        pagination: PagePaginationQuery { per_page, page },
        status: status_filter,
    } = query.into_inner();

    let mut conn = db.get_conn().await?;

    // FIXME: Preliminary solution, consider using UNION when Diesel supports it.
    // As in #[get("/events")], we simply get all invitees and truncate them afterwards.
    // Note that get_for_event_paginated returns a total record count of 0 when paging beyond the end.

    let (event_invites_with_user, event_invites_total) =
        EventInvite::get_for_event_paginated(&mut conn, event_id, i64::MAX, 1, status_filter)
            .await?;

    let event_invitees_iter = event_invites_with_user
        .into_iter()
        .map(|(event_invite, user)| {
            EventInvitee::from_invite_with_user(event_invite, user, &settings)
        });

    let (event_email_invites, event_email_invites_total) =
        EventEmailInvite::get_for_event_paginated(&mut conn, event_id, i64::MAX, 1).await?;

    drop(conn);

    let event_email_invitees_iter = event_email_invites
        .into_iter()
        .map(|event_email_invite| EventInvitee::from_email_invite(event_email_invite, &settings));

    let invitees_to_skip_count = (page - 1) * per_page;
    let invitees = event_invitees_iter
        .chain(event_email_invitees_iter)
        .skip(invitees_to_skip_count as usize)
        .take(per_page as usize)
        .collect();

    let invitees =
        enrich_invitees_from_keycloak(settings, &kc_admin_client, &current_tenant, invitees).await;

    Ok(ApiResponse::new(invitees).with_page_pagination(
        per_page,
        page,
        event_invites_total + event_email_invites_total,
    ))
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
    let event_id = event_id.into_inner();

    let send_email_notification = !query.suppress_email_notification;

    match create_invite.into_inner() {
        PostEventInviteBody::User(user_invite) => {
            create_user_event_invite(
                db,
                authz,
                current_user.into_inner(),
                event_id,
                user_invite,
                &mail_service.into_inner(),
                send_email_notification,
            )
            .await
        }
        PostEventInviteBody::Email(email_invite) => {
            create_email_event_invite(
                settings,
                db,
                authz,
                kc_admin_client,
                current_tenant.into_inner(),
                current_user.into_inner(),
                event_id,
                email_invite,
                &mail_service.into_inner(),
                send_email_notification,
            )
            .await
        }
    }
}

async fn create_user_event_invite(
    db: Data<Db>,
    authz: Data<Authz>,
    current_user: User,
    event_id: EventId,
    user_invite: UserInvite,
    mail_service: &MailService,
    send_email_notification: bool,
) -> Result<Either<Created, NoContent>, ApiError> {
    let inviter = current_user.clone();

    let mut conn = db.get_conn().await?;

    let (event, room, sip_config) = Event::get_with_room(&mut conn, event_id).await?;
    let invitee =
        User::get_filtered_by_tenant(&mut conn, event.tenant_id, user_invite.invitee).await?;
    let shared_folder = EventSharedFolder::get_for_event(&mut conn, event_id)
        .await?
        .map(SharedFolder::from);
    let streaming_targets = get_room_streaming_targets(&mut conn, room.id).await?;

    if event.created_by == user_invite.invitee {
        return Ok(Either::Right(NoContent));
    }

    let res = NewEventInvite {
        event_id,
        invitee: user_invite.invitee,
        role: user_invite.role,
        created_by: current_user.id,
        created_at: None,
    }
    .try_insert(&mut conn)
    .await?;

    drop(conn);

    match res {
        Some(_invite) => {
            let policies = PoliciesBuilder::new()
                // Grant invitee access
                .grant_user_access(invitee.id)
                .event_read_access(event_id)
                .room_read_access(event.room)
                .event_invite_invitee_access(event_id)
                .finish();

            authz.add_policies(policies).await?;

            if send_email_notification {
                mail_service
                    .send_registered_invite(
                        inviter,
                        event,
                        room,
                        sip_config,
                        invitee,
                        shared_folder,
                        streaming_targets,
                    )
                    .await
                    .map_err(|e| {
                        log::warn!("Failed to send with MailService: {}", Report::from_error(e));
                        ApiError::internal()
                    })?;
            }

            Ok(Either::Left(Created))
        }
        None => Ok(Either::Right(NoContent)),
    }
}

/// Create an invite to an event via email address
///
/// Checks first if a user exists with the email address in our database and creates a regular invite,
/// else checks if the email is registered with the keycloak (or external invitee support is configured)
/// and then creates an email invite
#[allow(clippy::too_many_arguments)]
async fn create_email_event_invite(
    settings: SharedSettingsActix,
    db: Data<Db>,
    authz: Data<Authz>,
    kc_admin_client: Data<KeycloakAdminClient>,
    current_tenant: Tenant,
    current_user: User,
    event_id: EventId,
    email_invite: EmailInvite,
    mail_service: &MailService,
    send_email_notification: bool,
) -> Result<Either<Created, NoContent>, ApiError> {
    let email = email_invite.email.to_lowercase();

    #[allow(clippy::large_enum_variant)]
    enum UserState {
        ExistsAndIsAlreadyInvited,
        ExistsAndWasInvited {
            event: Event,
            room: Room,
            invitee: User,
            sip_config: Option<SipConfig>,
            invite: EventInvite,
            shared_folder: Option<SharedFolder>,
            streaming_targets: Vec<RoomStreamingTarget>,
        },
        DoesNotExist {
            event: Event,
            room: Room,
            sip_config: Option<SipConfig>,
            shared_folder: Option<SharedFolder>,
            streaming_targets: Vec<RoomStreamingTarget>,
        },
    }

    let state = {
        let current_user = current_user.clone();
        let db = db.clone();

        let mut conn = db.get_conn().await?;

        let (event, room, sip_config) = Event::get_with_room(&mut conn, event_id).await?;
        let shared_folder = EventSharedFolder::get_for_event(&mut conn, event_id)
            .await?
            .map(SharedFolder::from);
        let streaming_targets = get_room_streaming_targets(&mut conn, room.id).await?;

        let invitee_user =
            User::get_by_email(&mut conn, current_user.tenant_id, email.as_ref()).await?;

        if let Some(invitee_user) = invitee_user {
            if event.created_by == invitee_user.id {
                UserState::ExistsAndIsAlreadyInvited
            } else {
                let res = NewEventInvite {
                    event_id,
                    invitee: invitee_user.id,
                    role: email_invite.role.into(),
                    created_by: current_user.id,
                    created_at: None,
                }
                .try_insert(&mut conn)
                .await?;

                match res {
                    Some(invite) => UserState::ExistsAndWasInvited {
                        event,
                        room,
                        invitee: invitee_user,
                        sip_config,
                        invite,
                        shared_folder,
                        streaming_targets,
                    },
                    None => UserState::ExistsAndIsAlreadyInvited,
                }
            }
        } else {
            UserState::DoesNotExist {
                event,
                room,
                sip_config,
                shared_folder,
                streaming_targets,
            }
        }
    };

    match state {
        UserState::ExistsAndIsAlreadyInvited => Ok(Either::Right(NoContent)),
        UserState::ExistsAndWasInvited {
            event,
            room,
            invite,
            sip_config,
            invitee,
            shared_folder,
            streaming_targets,
        } => {
            let policies = PoliciesBuilder::new()
                // Grant invitee access
                .grant_user_access(invite.invitee)
                .event_read_access(event_id)
                .room_read_access(room.id)
                .event_invite_invitee_access(event_id)
                .finish();

            authz.add_policies(policies).await?;

            if send_email_notification {
                mail_service
                    .send_registered_invite(
                        current_user,
                        event,
                        room,
                        sip_config,
                        invitee,
                        shared_folder,
                        streaming_targets,
                    )
                    .await
                    .map_err(|e| {
                        log::warn!("Failed to send with MailService: {}", Report::from_error(e));
                        ApiError::internal()
                    })?;
            }

            Ok(Either::Left(Created))
        }
        UserState::DoesNotExist {
            event,
            room,
            sip_config,
            shared_folder,
            streaming_targets,
        } => {
            create_invite_to_non_matching_email(
                settings,
                db,
                authz,
                kc_admin_client,
                mail_service,
                send_email_notification,
                current_tenant,
                current_user,
                event,
                room,
                sip_config,
                email,
                email_invite.role,
                shared_folder,
                streaming_targets,
            )
            .await
        }
    }
}

/// Invite a given email to the event.
/// Will check if the email exists in keycloak and sends an "unregistered" email invite
/// or (if configured) sends an "external" email invite to the given email address
#[allow(clippy::too_many_arguments)]
async fn create_invite_to_non_matching_email(
    settings: SharedSettingsActix,
    db: Data<Db>,
    authz: Data<Authz>,
    kc_admin_client: Data<KeycloakAdminClient>,
    mail_service: &MailService,
    send_email_notification: bool,
    current_tenant: Tenant,
    current_user: User,
    event: Event,
    room: Room,
    sip_config: Option<SipConfig>,
    email: EmailAddress,
    role: EmailInviteRole,
    shared_folder: Option<SharedFolder>,
    streaming_targets: Vec<RoomStreamingTarget>,
) -> Result<Either<Created, NoContent>, ApiError> {
    let settings = settings.load();

    let tenant_filter = get_tenant_filter(&current_tenant, &settings.tenants.assignment);

    let invitee_user = kc_admin_client
        .get_user_for_email(tenant_filter, email.as_ref())
        .await
        .map_err(|e| {
            log::warn!("Failed to query user for email: {}", Report::from_error(e));
            ApiError::internal()
        })?;

    if invitee_user.is_some() || settings.endpoints.event_invite_external_email_address {
        let inviter = current_user.clone();
        let invitee_email = email.clone();

        let mut conn = db.get_conn().await?;

        let res = {
            let event_id = event.id;
            let current_user_id = current_user.id;

            NewEventEmailInvite {
                event_id,
                email: email.into(),
                role,
                created_by: current_user_id,
            }
            .try_insert(&mut conn)
            .await?
        };

        match res {
            Some(_) => {
                if let (Some(invitee_user), true) = (invitee_user, send_email_notification) {
                    mail_service
                        .send_unregistered_invite(
                            inviter,
                            event,
                            room,
                            sip_config,
                            invitee_user,
                            shared_folder,
                            streaming_targets,
                        )
                        .await
                        .map_err(|e| {
                            log::warn!(
                                "Failed to send with MailService: {}",
                                Report::from_error(e)
                            );
                            ApiError::internal()
                        })?;
                } else {
                    let invite = NewInvite {
                        active: true,
                        created_by: current_user.id,
                        updated_by: current_user.id,
                        room: room.id,
                        expiration: None,
                    }
                    .insert(&mut conn)
                    .await?;

                    let policies = PoliciesBuilder::new()
                        // Grant invitee access
                        .grant_invite_access(invite.id)
                        .room_guest_read_access(room.id)
                        .finish();

                    authz.add_policies(policies).await?;

                    if send_email_notification {
                        mail_service
                            .send_external_invite(
                                inviter,
                                event,
                                room,
                                sip_config,
                                invitee_email.as_ref(),
                                invite.id.to_string(),
                                shared_folder,
                                streaming_targets,
                            )
                            .await
                            .map_err(|e| {
                                log::warn!(
                                    "Failed to send with MailService: {}",
                                    Report::from_error(e)
                                );
                                ApiError::internal()
                            })?;
                    }
                }

                Ok(Either::Left(Created))
            }
            None => Ok(Either::Right(NoContent)),
        }
    } else {
        Err(ApiError::conflict()
            .with_code("unknown_email")
            .with_message(
                "Only emails registered with the systems are allowed to be used for invites",
            ))
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
    db: Data<Db>,
    current_user: ReqData<User>,
    path_parameters: Path<(EventId, UserId)>,
    update_invite: Json<PatchInviteBody>,
) -> Result<NoContent, ApiError> {
    let (event_id, user_id) = path_parameters.into_inner();

    let mut conn = db.get_conn().await?;

    let event = Event::get(&mut conn, event_id).await?;

    if event.created_by != current_user.id {
        return Err(ApiError::forbidden());
    }

    let changeset = UpdateEventInvite {
        status: None,
        role: update_invite.role,
    };

    changeset.apply(&mut conn, user_id, event_id).await?;

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
    let event_id = path_parameters.into_inner();

    let mut conn = db.get_conn().await?;

    let event = Event::get(&mut conn, event_id).await?;

    if event.created_by != current_user.id {
        return Err(ApiError::forbidden());
    }

    let changeset = UpdateEventEmailInvite {
        role: update_invite.role,
    };

    changeset
        .apply(&mut conn, update_invite.email.to_string(), event_id)
        .await?;

    Ok(NoContent)
}

struct UninviteNotificationValues {
    pub tenant: Tenant,
    pub created_by: User,
    pub event: Event,
    pub room: Room,
    pub sip_config: Option<SipConfig>,
    pub users_to_notify: Vec<MailRecipient>,
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
    let settings = settings.load_full();
    let current_user = current_user.into_inner();

    let DeleteEventInvitePath { event_id, user_id } = path_params.into_inner();

    let send_email_notification = !query.suppress_email_notification;

    let mut conn = db.get_conn().await?;

    // TODO(w.rabl) Further DB access optimization (replacing call to get_with_invite_and_room)?
    let (event, _invite, room, sip_config, _is_favorite, shared_folder, _tariff) =
        Event::get_with_related_items(&mut conn, current_user.id, event_id).await?;
    let streaming_targets = get_room_streaming_targets(&mut conn, room.id).await?;

    let created_by = if event.created_by == current_user.id {
        current_user.clone()
    } else {
        User::get(&mut conn, event.created_by).await?
    };

    let invited_users = get_invited_mail_recipients_for_event(&mut conn, event_id).await?;

    let (room_id, invite) = conn
        .transaction(|conn| {
            async move {
                // delete invite to the event
                let invite = EventInvite::delete_by_invitee(conn, event_id, user_id).await?;

                // user access is going to be removed for the event, remove favorite entry if it exists
                EventFavorite::delete_by_id(conn, current_user.id, event_id).await?;

                let event = Event::get(conn, invite.event_id).await?;

                // TODO: type inference just dies here with this
                Ok((event.room, invite)) as opentalk_database::Result<(RoomId, EventInvite)>
            }
            .scope_boxed()
        })
        .await?;

    drop(conn);

    if send_email_notification {
        // Notify just the specified user. Currently, unlike the create_invite_to_event counterpart, this endpoint
        // only handles and notifies a single registered user. This somehow contradicts patch_event and delete_event
        // as well.
        // See this issue for more details: https://git.opentalk.dev/opentalk/backend/services/controller/-/issues/499.
        let users_to_notify: Vec<MailRecipient> = invited_users
            .into_iter()
            .filter(|user| match user {
                MailRecipient::Registered(user) => user.id == user_id,
                MailRecipient::Unregistered(_) => false,
                MailRecipient::External(_) => false,
            })
            .collect();

        let notification_values = UninviteNotificationValues {
            tenant: current_tenant.into_inner(),
            created_by,
            event,
            room,
            sip_config,
            users_to_notify,
        };

        notify_invitees_about_uninvite(
            settings,
            notification_values,
            mail_service.into_inner(),
            &kc_admin_client,
            shared_folder.map(SharedFolder::from),
            streaming_targets,
        )
        .await;
    }

    remove_invitee_permissions(&authz, event_id, room_id, invite.invitee).await?;

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
    let settings = settings.load_full();
    let current_user = current_user.into_inner();
    let current_tenant = current_tenant.into_inner();
    let event_id = path.into_inner();
    let email = body.into_inner().email.to_lowercase().to_string();
    let tenant_filter = get_tenant_filter(&current_tenant, &settings.tenants.assignment);

    let send_email_notification = !query.suppress_email_notification;

    let mut conn = db.get_conn().await?;

    let (event, _invite, room, sip_config, _is_favorite, shared_folder, _tariff) =
        Event::get_with_related_items(&mut conn, current_user.id, event_id).await?;
    let streaming_targets = get_room_streaming_targets(&mut conn, room.id).await?;

    let created_by = if event.created_by == current_user.id {
        current_user.clone()
    } else {
        User::get(&mut conn, event.created_by).await?
    };

    let user_from_db = User::get_by_email(&mut conn, current_tenant.id, &email).await?;

    let mail_recipient = if let Some(user) = user_from_db {
        let user_id = user.id;

        conn.transaction(|conn| {
            async move {
                // delete invite to the event
                log::error!("deleting: {event_id}, {user_id}");

                EventInvite::delete_by_invitee(conn, event_id, user_id).await?;

                // user access is going to be removed for the event, remove favorite entry if it exists
                EventFavorite::delete_by_id(conn, current_user.id, event_id).await?;

                Ok(()) as opentalk_database::Result<()>
            }
            .scope_boxed()
        })
        .await?;

        remove_invitee_permissions(&authz, event_id, room.id, user_id).await?;

        MailRecipient::Registered(RegisteredMailRecipient {
            email,
            ..user.into()
        })
    } else if let Ok(Some(user)) = kc_admin_client
        .get_user_for_email(tenant_filter, email.as_ref())
        .await
    {
        EventEmailInvite::delete(&mut conn, &event_id, &email).await?;

        MailRecipient::Unregistered(UnregisteredMailRecipient {
            email,
            first_name: user.first_name,
            last_name: user.last_name,
        })
    } else {
        EventEmailInvite::delete(&mut conn, &event_id, &email).await?;

        MailRecipient::External(ExternalMailRecipient { email })
    };

    if send_email_notification {
        let notification_values = UninviteNotificationValues {
            tenant: current_tenant,
            created_by,
            event,
            room,
            sip_config,
            users_to_notify: vec![mail_recipient],
        };

        notify_invitees_about_uninvite(
            settings,
            notification_values,
            mail_service.into_inner(),
            &kc_admin_client,
            shared_folder.map(SharedFolder::from),
            streaming_targets,
        )
        .await;
    }

    Ok(NoContent)
}

async fn remove_invitee_permissions(
    authz: &Authz,
    event_id: EventId,
    room_id: RoomId,
    user_id: UserId,
) -> Result<(), ApiError> {
    let resources = vec![
        format!("/events/{event_id}"),
        format!("/events/{event_id}/instances"),
        format!("/events/{event_id}/instances/*"),
        format!("/events/{event_id}/invites"),
        format!("/users/me/event_favorites/{event_id}"),
        format!("/events/{event_id}/invite"),
        format!("/events/{event_id}/shared_folder"),
        format!("/rooms/{room_id}"),
        format!("/rooms/{room_id}/invites"),
        format!("/rooms/{room_id}/start"),
        format!("/rooms/{room_id}/tariff"),
        format!("/rooms/{room_id}/event"),
        format!("/rooms/{room_id}/assets"),
        format!("/rooms/{room_id}/assets/*"),
    ];

    authz
        .remove_all_user_permission_for_resources(user_id, resources)
        .await?;

    Ok(())
}

/// Part of `DELETE /events/{event_id}/invites/{user_id}` (see [`delete_invite_to_event`])
///
/// Notify invited users about the event deletion
async fn notify_invitees_about_uninvite(
    settings: Arc<Settings>,
    notification_values: UninviteNotificationValues,
    mail_service: Arc<MailService>,
    kc_admin_client: &Data<KeycloakAdminClient>,
    shared_folder: Option<SharedFolder>,
    streaming_targets: Vec<RoomStreamingTarget>,
) {
    // Don't send mails for past events
    match notification_values.event.ends_at {
        Some(ends_at) if ends_at < Utc::now() => {
            return;
        }
        _ => {}
    }
    for user in notification_values.users_to_notify {
        let invited_user = enrich_from_keycloak(
            settings.clone(),
            user,
            &notification_values.tenant,
            kc_admin_client,
        )
        .await;

        if let Err(e) = mail_service
            .send_event_uninvite(
                notification_values.created_by.clone(),
                notification_values.event.clone(),
                notification_values.room.clone(),
                notification_values.sip_config.clone(),
                invited_user,
                shared_folder.clone(),
                streaming_targets.clone(),
            )
            .await
        {
            log::error!(
                "Failed to send event uninvite with MailService, {}",
                Report::from_error(e)
            );
        }
    }
}

/// API Endpoint `GET /users/me/pending_invites`
#[get("/users/me/pending_invites")]
pub async fn get_event_invites_pending(
    db: Data<Db>,
    current_user: ReqData<User>,
) -> DefaultApiResult<GetEventInvitesPendingResponse> {
    let mut conn = db.get_conn().await?;

    let event_invites = EventInvite::get_pending_for_user(&mut conn, current_user.id).await?;

    Ok(ApiResponse::new(GetEventInvitesPendingResponse {
        total_pending_invites: event_invites.len() as u32,
    }))
}

/// API Endpoint `PATCH /events/{event_id}/invite`
///
/// Accept an invite to an event
#[patch("/events/{event_id}/invite")]
pub async fn accept_event_invite(
    db: Data<Db>,
    current_user: ReqData<User>,
    event_id: Path<EventId>,
) -> Result<NoContent, ApiError> {
    let event_id = event_id.into_inner();

    let mut conn = db.get_conn().await?;

    let changeset = UpdateEventInvite {
        status: Some(EventInviteStatus::Accepted),
        role: None,
    };

    changeset
        .apply(&mut conn, current_user.id, event_id)
        .await?;

    Ok(NoContent)
}

/// API Endpoint `DELETE /events/{event_id}/invite`
///
/// Decline an invite to an event
#[delete("/events/{event_id}/invite")]
pub async fn decline_event_invite(
    db: Data<Db>,
    current_user: ReqData<User>,
    event_id: Path<EventId>,
) -> Result<NoContent, ApiError> {
    let event_id = event_id.into_inner();

    let mut conn = db.get_conn().await?;

    let changeset = UpdateEventInvite {
        status: Some(EventInviteStatus::Declined),
        role: None,
    };

    changeset
        .apply(&mut conn, current_user.id, event_id)
        .await?;

    Ok(NoContent)
}
