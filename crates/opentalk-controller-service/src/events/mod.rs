// SPDX-FileCopyrightText: OpenTalk GmbH <mail@opentalk.eu>
//
// SPDX-License-Identifier: EUPL-1.2

//! Provides events stuff.

pub(crate) mod notifications;

use opentalk_controller_settings::{settings_file::TenantAssignment, Settings};
use opentalk_database::DbConnection;
use opentalk_db_storage::{
    events::{email_invites::EventEmailInvite, shared_folders::EventSharedFolder, EventInvite},
    tenants::Tenant,
};
use opentalk_keycloak_admin::{users::TenantFilter, KeycloakAdminClient};
use opentalk_types_api_v1::{
    events::{EventInvitee, EventInviteeProfile},
    users::UnregisteredUser,
};
use opentalk_types_common::{
    events::EventId,
    shared_folders::{SharedFolder, SharedFolderAccess},
    users::UserId,
};

use crate::services::{ExternalMailRecipient, MailRecipient, UnregisteredMailRecipient};

/// Gets the invited mail recipients for an event
pub async fn get_invited_mail_recipients_for_event(
    conn: &mut DbConnection,
    event_id: EventId,
) -> opentalk_database::Result<Vec<MailRecipient>> {
    // TODO(w.rabl) Further DB access optimization (replacing call to get_for_event_paginated)?
    let (invites_with_user, _) =
        EventInvite::get_for_event_paginated(conn, event_id, i64::MAX, 1, None).await?;
    let user_invitees = invites_with_user
        .into_iter()
        .map(|(_, user)| MailRecipient::Registered(user.into()));

    let (email_invites, _) =
        EventEmailInvite::get_for_event_paginated(conn, event_id, i64::MAX, 1).await?;
    let email_invitees = email_invites.into_iter().map(|invitee| {
        MailRecipient::External(ExternalMailRecipient {
            email: invitee.email,
        })
    });

    let invitees = user_invitees.chain(email_invitees).collect();

    Ok(invitees)
}

/// Gets some additional information about invitees from Keycloak and attaches it to them
pub async fn enrich_invitees_from_keycloak(
    settings: &Settings,
    kc_admin_client: &KeycloakAdminClient,
    current_tenant: &Tenant,
    invitees: Vec<EventInvitee>,
) -> Vec<EventInvitee> {
    let tenant_assignment = &settings.raw.tenants.assignment;
    let invitee_mapping_futures = invitees.into_iter().map(|invitee| async move {
        if let EventInviteeProfile::Email(profile_details) = invitee.profile {
            let tenant_filter = get_tenant_filter(current_tenant, tenant_assignment);

            let user_for_email = kc_admin_client
                .get_user_for_email(tenant_filter, profile_details.email.as_ref())
                .await
                .unwrap_or_default();

            if let Some(user) = user_for_email {
                let profile_details = UnregisteredUser {
                    email: profile_details.email,
                    firstname: user.first_name,
                    lastname: user.last_name,
                    avatar_url: profile_details.avatar_url,
                };
                EventInvitee {
                    profile: EventInviteeProfile::Unregistered(profile_details),
                    ..invitee
                }
            } else {
                EventInvitee {
                    profile: EventInviteeProfile::Email(profile_details),
                    ..invitee
                }
            }
        } else {
            invitee
        }
    });
    futures::future::join_all(invitee_mapping_futures).await
}

/// Gets a tenat filter
pub fn get_tenant_filter<'a>(
    current_tenant: &'a Tenant,
    tenant_assignment: &'a TenantAssignment,
) -> Option<TenantFilter<'a>> {
    match tenant_assignment {
        TenantAssignment::Static { .. } => None,
        TenantAssignment::ByExternalTenantId {
            external_tenant_id_user_attribute_name,
        } => Some(TenantFilter {
            field_name: external_tenant_id_user_attribute_name,
            id: current_tenant.oidc_tenant_id.as_ref(),
        }),
    }
}

/// Gets the share folder of a user
pub fn shared_folder_for_user(
    shared_folder: Option<EventSharedFolder>,
    event_created_by: UserId,
    current_user: UserId,
) -> Option<SharedFolder> {
    shared_folder.map(|f| {
        let EventSharedFolder {
            write_password,
            write_url,
            read_password,
            read_url,
            ..
        } = f;

        let read_write = if event_created_by == current_user {
            Some(SharedFolderAccess {
                url: write_url,
                password: write_password,
            })
        } else {
            None
        };

        let read = SharedFolderAccess {
            url: read_url,
            password: read_password,
        };

        SharedFolder { read, read_write }
    })
}

/// Gets some additional information about invitees from Keycloak and attaches it to them
pub async fn enrich_from_keycloak(
    settings: &Settings,
    recipient: MailRecipient,
    current_tenant: &Tenant,
    kc_admin_client: &KeycloakAdminClient,
) -> MailRecipient {
    let tenant_assignment = &settings.raw.tenants.assignment;
    if let MailRecipient::External(recipient) = recipient {
        let tenant_filter = get_tenant_filter(current_tenant, tenant_assignment);

        let keycloak_user = kc_admin_client
            .get_user_for_email(tenant_filter, recipient.email.as_ref())
            .await
            .unwrap_or_default();

        if let Some(keycloak_user) = keycloak_user {
            MailRecipient::Unregistered(UnregisteredMailRecipient {
                email: recipient.email,
                first_name: keycloak_user.first_name,
                last_name: keycloak_user.last_name,
            })
        } else {
            MailRecipient::External(recipient)
        }
    } else {
        recipient
    }
}
