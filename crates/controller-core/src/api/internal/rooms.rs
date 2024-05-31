// SPDX-FileCopyrightText: OpenTalk GmbH <mail@opentalk.eu>
//
// SPDX-License-Identifier: EUPL-1.2

use actix_web::{
    delete,
    web::{Data, Path, Query, ReqData},
};
use kustos::prelude::*;
use opentalk_controller_utils::deletion::{Deleter as _, RoomDeleter};
use opentalk_database::{Db, DbConnection};
use opentalk_db_storage::{events::Event, tenants::Tenant, users::User};
use opentalk_keycloak_admin::KeycloakAdminClient;
use opentalk_signaling_core::{ExchangeHandle, ObjectStorage};
use opentalk_types::{
    api::{error::ApiError, v1::rooms::DeleteRoomQuery},
    common::shared_folder::SharedFolder,
    core::RoomId,
};

use crate::{
    api::{
        internal::NoContent,
        v1::events::{
            get_invited_mail_recipients_for_event, notify_invitees_about_delete,
            CancellationNotificationValues,
        },
    },
    services::{MailRecipient, MailService},
    settings::SharedSettingsActix,
};

/// API Endpoint *DELETE /rooms/{room_id}*
///
/// Deletes the room and owned resources and linked events. This endpoint is rather complex as it
/// deletes multiple underlying REST exposed resources.
/// We need to check if we have access to all resources that need to be removed during this operation, and
/// we need to make sure to delete all related authz permissions of those resources.
///
/// We cannot rely on DB cascading as this would result in idling permissions.
///
/// Important:
/// Access checks should not be handled via a middleware but instead done inside, as this deletes multiple resources
#[delete("/rooms/{room_id}")]
#[allow(clippy::too_many_arguments)]
pub async fn delete(
    settings: SharedSettingsActix,
    db: Data<Db>,
    storage: Data<ObjectStorage>,
    exchange_handle: Data<ExchangeHandle>,
    room_id: Path<RoomId>,
    current_user: ReqData<User>,
    current_tenant: ReqData<Tenant>,
    authz: Data<Authz>,
    query: Query<DeleteRoomQuery>,
    mail_service: Data<MailService>,
    kc_admin_client: Data<KeycloakAdminClient>,
) -> Result<NoContent, ApiError> {
    let room_id = room_id.into_inner();
    let current_user = current_user.into_inner();
    let current_tenant = current_tenant.into_inner();
    let settings = settings.load_full();
    let mail_service = mail_service.into_inner();

    let mut conn = db.get_conn().await?;

    let notification_values = if !query.suppress_email_notification {
        gather_mail_notification_values(&mut conn, &current_user, &current_tenant, room_id).await?
    } else {
        None
    };

    let deleter = RoomDeleter::new(room_id, false);

    deleter
        .perform(
            log::logger(),
            &mut conn,
            &authz,
            Some(current_user.id),
            exchange_handle.as_ref().clone(),
            &settings,
            &storage,
        )
        .await?;

    if let Some(notification_values) = notification_values {
        notify_invitees_about_delete(
            settings,
            notification_values,
            mail_service,
            &kc_admin_client,
        )
        .await;
    }

    Ok(NoContent {})
}

async fn gather_mail_notification_values(
    conn: &mut DbConnection,
    current_user: &User,
    current_tenant: &Tenant,
    room_id: RoomId,
) -> Result<Option<CancellationNotificationValues>, ApiError> {
    let linked_event_id = match Event::get_id_for_room(conn, room_id).await? {
        Some(event) => event,
        None => return Ok(None),
    };

    let (event, _invite, room, sip_config, _is_favorite, shared_folder, _tariff) =
        Event::get_with_related_items(conn, current_user.id, linked_event_id).await?;

    let invitees = get_invited_mail_recipients_for_event(conn, event.id).await?;
    let created_by_mail_recipient = MailRecipient::Registered(current_user.clone().into());

    let users_to_notify = invitees
        .into_iter()
        .chain(std::iter::once(created_by_mail_recipient))
        .collect::<Vec<_>>();

    let notification_values = CancellationNotificationValues {
        tenant: current_tenant.clone(),
        created_by: current_user.clone(),
        event,
        room,
        sip_config,
        users_to_notify,
        shared_folder: shared_folder.map(SharedFolder::from),
    };

    Ok(Some(notification_values))
}
