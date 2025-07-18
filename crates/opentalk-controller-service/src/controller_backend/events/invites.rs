// SPDX-FileCopyrightText: OpenTalk GmbH <mail@opentalk.eu>
//
// SPDX-License-Identifier: EUPL-1.2

//! Handles event invites

use chrono::Utc;
use diesel_async::{AsyncConnection, scoped_futures::ScopedFutureExt};
use kustos::{Authz, policies_builder::PoliciesBuilder};
use opentalk_controller_service_facade::RequestUser;
use opentalk_controller_settings::Settings;
use opentalk_controller_utils::CaptureApiError;
use opentalk_database::{DatabaseError, Db};
use opentalk_db_storage::{
    events::{
        Event, EventFavorite, EventInvite, NewEventInvite, UpdateEventInvite,
        email_invites::{EventEmailInvite, NewEventEmailInvite, UpdateEventEmailInvite},
        shared_folders::EventSharedFolder,
    },
    invites::NewInvite,
    rooms::Room,
    sip_configs::SipConfig,
    streaming_targets::get_room_streaming_targets,
    tenants::Tenant,
    users::User,
};
use opentalk_keycloak_admin::KeycloakAdminClient;
use opentalk_types_api_v1::{
    error::ApiError,
    events::{
        DeleteEventInvitePath, EmailInvite, EventInvitee, EventOptionsQuery, PatchEmailInviteBody,
        PatchInviteBody, PostEventInviteBody, PostEventInviteQuery, UserInvite,
        by_event_id::invites::GetEventsInvitesQuery,
    },
    pagination::PagePaginationQuery,
    users::GetEventInvitesPendingResponseBody,
};
use opentalk_types_common::{
    email::EmailAddress,
    events::{
        EventId,
        invites::{EmailInviteRole, EventInviteStatus},
    },
    rooms::RoomId,
    shared_folders::SharedFolder,
    streaming::RoomStreamingTarget,
    users::UserId,
};
use snafu::Report;

use crate::{
    ControllerBackend,
    controller_backend::{
        RoomsPoliciesBuilderExt,
        events::{EventInviteeExt, EventPoliciesBuilderExt},
    },
    events::{
        enrich_from_optional_user_search, enrich_invitees_from_optional_user_search,
        get_invited_mail_recipients_for_event, get_tenant_filter,
    },
    services::{
        ExternalMailRecipient, MailRecipient, MailService, RegisteredMailRecipient,
        UnregisteredMailRecipient,
    },
};

impl ControllerBackend {
    pub(crate) async fn get_invites_for_event(
        &self,
        current_user: RequestUser,
        event_id: EventId,
        GetEventsInvitesQuery {
            pagination: PagePaginationQuery { per_page, page },
            status: status_filter,
        }: GetEventsInvitesQuery,
    ) -> Result<(Vec<EventInvitee>, i64, i64, i64), CaptureApiError> {
        let settings = self.settings_provider.get();
        let mut conn = self.db.get_conn().await?;

        // FIXME: Preliminary solution, consider using UNION when Diesel supports it.
        // As in #[get("/events")], we simply get all invitees and truncate them afterwards.
        // Note that get_for_event_paginated returns a total record count of 0 when paging beyond the end.

        let (event_invites_with_user, event_invites_total) =
            EventInvite::get_for_event_paginated(&mut conn, event_id, i64::MAX, 1, status_filter)
                .await?;

        let event_invitees_iter =
            event_invites_with_user
                .into_iter()
                .map(|(event_invite, user)| {
                    EventInvitee::from_invite_with_user(event_invite, user, &settings)
                });

        let (event_email_invites, event_email_invites_total) =
            EventEmailInvite::get_for_event_paginated(&mut conn, event_id, i64::MAX, 1).await?;

        let current_tenant = Tenant::get(&mut conn, current_user.tenant_id).await?;

        drop(conn);

        let event_email_invitees_iter = event_email_invites.into_iter().map(|event_email_invite| {
            EventInvitee::from_email_invite(event_email_invite, &settings)
        });

        let invitees_to_skip_count = (page - 1) * per_page;
        let invitees = event_invitees_iter
            .chain(event_email_invitees_iter)
            .skip(invitees_to_skip_count as usize)
            .take(per_page as usize)
            .collect();

        let invitees = enrich_invitees_from_optional_user_search(
            &settings,
            &self.user_search_client,
            &current_tenant,
            invitees,
        )
        .await;

        Ok((
            invitees,
            per_page,
            page,
            event_invites_total + event_email_invites_total,
        ))
    }

    pub(crate) async fn create_invite_to_event(
        &self,
        current_user: RequestUser,
        event_id: EventId,
        query: PostEventInviteQuery,
        create_invite: PostEventInviteBody,
    ) -> Result<bool, CaptureApiError> {
        let settings = self.settings_provider.get();
        let mut conn = self.db.get_conn().await?;

        let mail_service = (!query.suppress_email_notification)
            .then(|| self.mail_service.as_ref().clone())
            .flatten();

        let current_tenant = Tenant::get(&mut conn, current_user.tenant_id).await?;
        let current_user = User::get(&mut conn, current_user.id).await?;

        match create_invite {
            PostEventInviteBody::User(user_invite) => {
                create_user_event_invite(
                    &settings,
                    &self.db,
                    &self.authz,
                    current_user,
                    event_id,
                    user_invite,
                    &mail_service,
                )
                .await
            }
            PostEventInviteBody::Email(email_invite) => {
                create_email_event_invite(
                    &settings,
                    &self.db,
                    &self.authz,
                    &self.user_search_client,
                    &current_tenant,
                    &current_user,
                    event_id,
                    email_invite,
                    &mail_service,
                )
                .await
            }
        }
    }

    pub(crate) async fn update_invite_to_event(
        &self,
        current_user: &RequestUser,
        event_id: EventId,
        user_id: UserId,
        update_invite: &PatchInviteBody,
    ) -> Result<(), CaptureApiError> {
        let mut conn = self.db.get_conn().await?;

        let event = Event::get(&mut conn, event_id).await?;

        if event.created_by != current_user.id {
            return Err(ApiError::forbidden().into());
        }

        let changeset = UpdateEventInvite {
            status: None,
            role: update_invite.role,
        };

        _ = changeset.apply(&mut conn, user_id, event_id).await?;

        Ok(())
    }

    pub(crate) async fn update_email_invite_to_event(
        &self,
        current_user: &RequestUser,
        event_id: EventId,
        update_invite: &PatchEmailInviteBody,
    ) -> Result<(), CaptureApiError> {
        let mut conn = self.db.get_conn().await?;

        let event = Event::get(&mut conn, event_id).await?;

        if event.created_by != current_user.id {
            return Err(ApiError::forbidden().into());
        }

        let changeset = UpdateEventEmailInvite {
            role: update_invite.role,
        };

        _ = changeset
            .apply(&mut conn, update_invite.email.as_ref(), event_id)
            .await?;

        Ok(())
    }

    pub(crate) async fn delete_invite_to_event(
        &self,
        current_user: RequestUser,
        DeleteEventInvitePath { event_id, user_id }: DeleteEventInvitePath,
        query: EventOptionsQuery,
    ) -> Result<(), CaptureApiError> {
        let settings = self.settings_provider.get();

        let mail_service = (!query.suppress_email_notification)
            .then(|| self.mail_service.as_ref().clone())
            .flatten();
        let mut conn = self.db.get_conn().await?;

        // TODO(w.rabl) Further DB access optimization (replacing call to get_with_invite_and_room)?
        let (
            event,
            _invite,
            room,
            sip_config,
            _is_favorite,
            shared_folder,
            _tariff,
            _training_participation_report_parameter_set,
        ) = Event::get_with_related_items(&mut conn, current_user.id, event_id).await?;
        let streaming_targets = get_room_streaming_targets(&mut conn, room.id).await?;

        let current_tenant = Tenant::get(&mut conn, current_user.tenant_id).await?;
        let current_user = User::get(&mut conn, current_user.id).await?;

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
                    _ = EventFavorite::delete_by_id(conn, current_user.id, event_id).await?;

                    let event = Event::get(conn, invite.event_id).await?;

                    // TODO: type inference just dies here with this
                    Ok::<(RoomId, EventInvite), DatabaseError>((event.room, invite))
                }
                .scope_boxed()
            })
            .await?;

        drop(conn);

        if let Some(mail_service) = &mail_service {
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
                tenant: current_tenant,
                created_by,
                event,
                room,
                sip_config,
                users_to_notify,
            };

            notify_invitees_about_uninvite(
                &settings,
                notification_values,
                mail_service,
                &self.user_search_client,
                shared_folder.map(SharedFolder::from),
                streaming_targets,
            )
            .await;
        }

        remove_invitee_permissions(&self.authz, event_id, room_id, invite.invitee).await?;

        Ok(())
    }

    pub(crate) async fn delete_email_invite_to_event(
        &self,
        current_user: RequestUser,
        event_id: EventId,
        email: EmailAddress,
        query: EventOptionsQuery,
    ) -> Result<(), CaptureApiError> {
        let settings = self.settings_provider.get();
        let mut conn = self.db.get_conn().await?;

        let email = email.to_lowercase().to_string();

        let current_tenant = Tenant::get(&mut conn, current_user.tenant_id).await?;
        let current_user = User::get(&mut conn, current_user.id).await?;

        let tenant_filter = get_tenant_filter(&current_tenant, &settings.tenants.assignment);

        let mail_service = (!query.suppress_email_notification)
            .then(|| self.mail_service.as_ref().clone())
            .flatten();

        let (
            event,
            _invite,
            room,
            sip_config,
            _is_favorite,
            shared_folder,
            _tariff,
            _training_participation_report_parameter_set,
        ) = Event::get_with_related_items(&mut conn, current_user.id, event_id).await?;
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

                    _ = EventInvite::delete_by_invitee(conn, event_id, user_id).await?;

                    // user access is going to be removed for the event, remove favorite entry if it exists
                    _ = EventFavorite::delete_by_id(conn, current_user.id, event_id).await?;

                    Ok::<(), DatabaseError>(())
                }
                .scope_boxed()
            })
            .await?;

            remove_invitee_permissions(&self.authz, event_id, room.id, user_id).await?;

            MailRecipient::Registered(RegisteredMailRecipient {
                email,
                ..user.into()
            })
        } else if let Ok(Some(user)) = {
            if let Some(user_search_client) = &*self.user_search_client {
                user_search_client
                    .get_user_for_email(tenant_filter, email.as_ref())
                    .await
            } else {
                Ok(None)
            }
        } {
            _ = EventEmailInvite::delete(&mut conn, &event_id, &email).await?;

            MailRecipient::Unregistered(UnregisteredMailRecipient {
                email,
                first_name: user.first_name,
                last_name: user.last_name,
            })
        } else {
            _ = EventEmailInvite::delete(&mut conn, &event_id, &email).await?;

            MailRecipient::External(ExternalMailRecipient { email })
        };

        if let Some(mail_service) = &mail_service {
            let notification_values = UninviteNotificationValues {
                tenant: current_tenant,
                created_by,
                event,
                room,
                sip_config,
                users_to_notify: vec![mail_recipient],
            };

            notify_invitees_about_uninvite(
                &settings,
                notification_values,
                mail_service,
                &self.user_search_client,
                shared_folder.map(SharedFolder::from),
                streaming_targets,
            )
            .await;
        }

        Ok(())
    }

    pub(crate) async fn get_event_invites_pending(
        &self,
        user_id: UserId,
    ) -> Result<GetEventInvitesPendingResponseBody, CaptureApiError> {
        let mut conn = self.db.get_conn().await?;

        let event_invites = EventInvite::get_pending_for_user(&mut conn, user_id).await?;

        Ok(GetEventInvitesPendingResponseBody {
            total_pending_invites: event_invites.len() as u32,
        })
    }

    pub(crate) async fn accept_event_invite(
        &self,
        user_id: UserId,
        event_id: EventId,
    ) -> Result<(), CaptureApiError> {
        let mut conn = self.db.get_conn().await?;

        let changeset = UpdateEventInvite {
            status: Some(EventInviteStatus::Accepted),
            role: None,
        };

        _ = changeset.apply(&mut conn, user_id, event_id).await?;

        Ok(())
    }

    pub(crate) async fn decline_event_invite(
        &self,
        user_id: UserId,
        event_id: EventId,
    ) -> Result<(), CaptureApiError> {
        let mut conn = self.db.get_conn().await?;

        let changeset = UpdateEventInvite {
            status: Some(EventInviteStatus::Declined),
            role: None,
        };

        _ = changeset.apply(&mut conn, user_id, event_id).await?;

        Ok(())
    }
}

#[allow(clippy::too_many_arguments)]
async fn create_user_event_invite(
    settings: &Settings,
    db: &Db,
    authz: &Authz,
    inviter: User,
    event_id: EventId,
    user_invite: UserInvite,
    mail_service: &Option<MailService>,
) -> Result<bool, CaptureApiError> {
    let mut conn = db.get_conn().await?;

    let (event, room, sip_config) = Event::get_with_room(&mut conn, event_id).await?;
    let invitee =
        User::get_filtered_by_tenant(&mut conn, event.tenant_id, user_invite.invitee).await?;
    let shared_folder = EventSharedFolder::get_for_event(&mut conn, event_id)
        .await?
        .map(SharedFolder::from);
    let streaming_targets = get_room_streaming_targets(&mut conn, room.id).await?;

    if event.created_by == user_invite.invitee {
        return Ok(false);
    }

    let res = NewEventInvite {
        event_id,
        invitee: user_invite.invitee,
        role: user_invite.role,
        created_by: inviter.id,
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

            if let Some(mail_service) = mail_service {
                mail_service
                    .send_registered_invite(
                        settings,
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

            Ok(true)
        }
        None => Ok(false),
    }
}

/// Create an invite to an event via email address
///
/// Checks first if a user exists with the email address in our database and creates a regular invite,
/// else checks if the email is registered with the Keycloak (or external invitee support is configured)
/// and then creates an email invite
#[allow(clippy::too_many_arguments)]
async fn create_email_event_invite(
    settings: &Settings,
    db: &Db,
    authz: &Authz,
    user_search_client: &Option<KeycloakAdminClient>,
    current_tenant: &Tenant,
    current_user: &User,
    event_id: EventId,
    email_invite: EmailInvite,
    mail_service: &Option<MailService>,
) -> Result<bool, CaptureApiError> {
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
        UserState::ExistsAndIsAlreadyInvited => Ok(false),
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

            if let Some(mail_service) = mail_service {
                mail_service
                    .send_registered_invite(
                        settings,
                        current_user.clone(),
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

            Ok(true)
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
                user_search_client,
                mail_service,
                current_tenant,
                current_user.clone(),
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
/// Will check if the email exists in Keycloak and sends an "unregistered" email invite
/// or (if configured) sends an "external" email invite to the given email address
#[allow(clippy::too_many_arguments)]
async fn create_invite_to_non_matching_email(
    settings: &Settings,
    db: &Db,
    authz: &Authz,
    user_search_client: &Option<KeycloakAdminClient>,
    mail_service: &Option<MailService>,
    current_tenant: &Tenant,
    current_user: User,
    event: Event,
    room: Room,
    sip_config: Option<SipConfig>,
    email: EmailAddress,
    role: EmailInviteRole,
    shared_folder: Option<SharedFolder>,
    streaming_targets: Vec<RoomStreamingTarget>,
) -> Result<bool, CaptureApiError> {
    let tenant_filter = get_tenant_filter(current_tenant, &settings.tenants.assignment);

    let invitee_user = if let Some(user_search_client) = user_search_client {
        user_search_client
            .get_user_for_email(tenant_filter, email.as_ref())
            .await
            .map_err(|e| {
                log::error!("Failed to query user for email: {}", Report::from_error(e));
                ApiError::internal()
            })?
    } else {
        None
    };

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
                if let (Some(invitee_user), Some(mail_service)) = (invitee_user, mail_service) {
                    mail_service
                        .send_unregistered_invite(
                            settings,
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

                    if let Some(mail_service) = mail_service {
                        mail_service
                            .send_external_invite(
                                settings,
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

                Ok(true)
            }
            None => Ok(false),
        }
    } else {
        Err(ApiError::conflict()
            .with_code("unknown_email")
            .with_message(
                "Only emails registered with the systems are allowed to be used for invites",
            )
            .into())
    }
}

struct UninviteNotificationValues {
    pub tenant: Tenant,
    pub created_by: User,
    pub event: Event,
    pub room: Room,
    pub sip_config: Option<SipConfig>,
    pub users_to_notify: Vec<MailRecipient>,
}

async fn remove_invitee_permissions(
    authz: &Authz,
    event_id: EventId,
    room_id: RoomId,
    user_id: UserId,
) -> Result<(), CaptureApiError> {
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
        format!("/rooms/{room_id}/streaming_targets"),
        format!("/rooms/{room_id}/roomserver/start"),
    ];

    _ = authz
        .remove_all_user_permission_for_resources(user_id, resources)
        .await?;

    Ok(())
}

/// Part of `DELETE /events/{event_id}/invites/{user_id}` (see [`delete_invite_to_event`])
///
/// Notify invited users about the event deletion
async fn notify_invitees_about_uninvite(
    settings: &Settings,
    notification_values: UninviteNotificationValues,
    mail_service: &MailService,
    user_search_client: &Option<KeycloakAdminClient>,
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
        let invited_user = enrich_from_optional_user_search(
            settings,
            user,
            &notification_values.tenant,
            user_search_client,
        )
        .await;

        if let Err(e) = mail_service
            .send_event_uninvite(
                settings,
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
