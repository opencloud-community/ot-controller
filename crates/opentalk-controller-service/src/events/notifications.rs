// SPDX-FileCopyrightText: OpenTalk GmbH <mail@opentalk.eu>
//
// SPDX-License-Identifier: EUPL-1.2

//! Handles event-related notifications.

use opentalk_controller_settings::Settings;
use opentalk_controller_utils::CaptureApiError;
use opentalk_database::DbConnection;
use opentalk_db_storage::{
    events::{Event, EventException},
    invites::Invite,
    rooms::Room,
    sip_configs::SipConfig,
    streaming_targets::get_room_streaming_targets,
    tenants::Tenant,
    users::User,
};
use opentalk_keycloak_admin::KeycloakAdminClient;
use opentalk_types_common::{
    rooms::RoomId, shared_folders::SharedFolder, streaming::RoomStreamingTarget,
};
use snafu::Report;

use crate::{
    events::{enrich_from_keycloak, get_invited_mail_recipients_for_event, shared_folder_for_user},
    services::{MailRecipient, MailService},
};

/// Provides information for event update notifications (e.g. via email)
#[derive(Debug)]
pub struct UpdateNotificationValues {
    /// The tenant id
    pub tenant: Tenant,
    /// The user who has created the event
    pub created_by: User,
    /// The event that was updated
    pub event: Event,
    /// The event exception that was updated
    pub event_exception: Option<EventException>,
    /// The room of the updated event
    pub room: Room,
    /// The SIP configuration of the updated event
    pub sip_config: Option<SipConfig>,
    /// The users to notify about the update
    pub users_to_notify: Vec<MailRecipient>,
    /// The updated invite
    pub invite_for_room: Invite,
}

/// Notifies the invitees of an event belonging to the specified room
pub async fn notify_event_invitees_by_room_about_update(
    kc_admin_client: &KeycloakAdminClient,
    settings: &Settings,
    mail_service: &MailService,
    current_tenant: Tenant,
    current_user: User,
    conn: &mut DbConnection,
    room_id: RoomId,
) -> Result<(), CaptureApiError> {
    let event = Event::get_for_room(conn, room_id).await?;

    if let Some(event) = event {
        let (
            event,
            _invite,
            room,
            sip_config,
            _is_favorite,
            shared_folder,
            _tariff,
            _training_participation_report,
        ) = Event::get_with_related_items(conn, current_user.id, event.id).await?;

        let shared_folder_for_user =
            shared_folder_for_user(shared_folder, event.created_by, current_user.id);

        let streaming_targets = get_room_streaming_targets(conn, room.id).await?;

        notify_event_invitees_about_update(
            kc_admin_client,
            settings,
            mail_service,
            current_tenant,
            current_user,
            conn,
            event,
            room,
            sip_config,
            shared_folder_for_user,
            streaming_targets,
        )
        .await?;
    }
    Ok(())
}

/// Notifies the invitees of an event about updates
#[allow(clippy::too_many_arguments)]
pub async fn notify_event_invitees_about_update(
    kc_admin_client: &KeycloakAdminClient,
    settings: &Settings,
    mail_service: &MailService,
    current_tenant: Tenant,
    current_user: User,
    conn: &mut DbConnection,
    event: Event,
    room: Room,
    sip_config: Option<SipConfig>,
    shared_folder_for_user: Option<SharedFolder>,
    streaming_targets: Vec<RoomStreamingTarget>,
) -> Result<(), CaptureApiError> {
    let invited_users = get_invited_mail_recipients_for_event(conn, event.id).await?;
    let current_user_mail_recipient = MailRecipient::Registered(current_user.clone().into());
    let users_to_notify = invited_users
        .into_iter()
        .chain(std::iter::once(current_user_mail_recipient))
        .collect::<Vec<_>>();
    let invite_for_room =
        Invite::get_first_or_create_for_room(conn, room.id, current_user.id).await?;
    let created_by = if event.created_by == current_user.id {
        current_user
    } else {
        User::get(conn, event.created_by).await?
    };

    let notification_values = UpdateNotificationValues {
        tenant: current_tenant,
        created_by,
        event,
        event_exception: None,
        room,
        sip_config,
        users_to_notify,
        invite_for_room,
    };

    notify_invitees_about_update(
        settings,
        notification_values,
        mail_service,
        kc_admin_client,
        shared_folder_for_user,
        streaming_targets,
    )
    .await;
    Ok(())
}

/// Notifies the invitees of an event about updates
pub async fn notify_invitees_about_update(
    settings: &Settings,
    notification_values: UpdateNotificationValues,
    mail_service: &MailService,
    kc_admin_client: &KeycloakAdminClient,
    shared_folder: Option<SharedFolder>,
    streaming_targets: Vec<RoomStreamingTarget>,
) {
    for user in notification_values.users_to_notify {
        let invited_user =
            enrich_from_keycloak(settings, user, &notification_values.tenant, kc_admin_client)
                .await;

        if let Err(e) = mail_service
            .send_event_update(
                settings,
                notification_values.created_by.clone(),
                notification_values.event.clone(),
                notification_values.event_exception.clone(),
                notification_values.room.clone(),
                notification_values.sip_config.clone(),
                invited_user,
                notification_values.invite_for_room.id.to_string(),
                shared_folder.clone(),
                streaming_targets.clone(),
            )
            .await
        {
            log::error!(
                "Failed to send event update with MailService, {}",
                Report::from_error(e)
            );
        }
    }
}
