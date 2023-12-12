// SPDX-FileCopyrightText: OpenTalk GmbH <mail@opentalk.eu>
//
// SPDX-License-Identifier: EUPL-1.2

use std::collections::HashSet;

use actix_http::StatusCode;
use actix_web::{
    delete, get, put,
    web::{Data, Json, Path, Query, ReqData},
    CustomizeResponder, Responder as _,
};
use anyhow::Result;
use chrono::{Days, NaiveDate, Utc};
use database::Db;
use db_storage::{
    events::{
        shared_folders::{EventSharedFolder, NewEventSharedFolder},
        Event,
    },
    users::User,
};
use log::warn;
use nextcloud_client::{Client, ShareId, SharePermission, ShareType};
use types::{
    api::v1::events::DeleteQuery,
    common::shared_folder::{SharedFolder, SharedFolderAccess},
    core::EventId,
};

use crate::{
    api::v1::response::{ApiError, NoContent},
    settings::SharedSettingsActix,
};

/// API Endpoint `GET /events/{event_id}/shared_folder`
///
/// Get the shared folder for an event
#[get("/events/{event_id}/shared_folder")]
pub async fn get_shared_folder_for_event(
    db: Data<Db>,
    event_id: Path<EventId>,
    current_user: ReqData<User>,
) -> Result<Json<SharedFolder>, ApiError> {
    let event_id = event_id.into_inner();
    let current_user = current_user.into_inner();

    let mut conn = db.get_conn().await?;

    let event = Event::get(&mut conn, event_id).await?;

    let shared_folder = SharedFolder::from(
        EventSharedFolder::get_for_event(&mut conn, event_id)
            .await?
            .ok_or_else(ApiError::not_found)?,
    );

    let shared_folder = if event.created_by == current_user.id {
        shared_folder
    } else {
        shared_folder.without_write_access()
    };

    Ok(Json(shared_folder))
}

#[put("/events/{event_id}/shared_folder")]
pub async fn put_shared_folder_for_event(
    settings: SharedSettingsActix,
    db: Data<Db>,
    event_id: Path<EventId>,
) -> Result<CustomizeResponder<Json<SharedFolder>>, ApiError> {
    let event_id = event_id.into_inner();

    let mut conn = db.get_conn().await?;
    let shared_folder = EventSharedFolder::get_for_event(&mut conn, event_id).await?;

    if let Some(shared_folder) = shared_folder {
        Ok(Json(SharedFolder::from(shared_folder))
            .customize()
            .with_status(StatusCode::OK))
    } else {
        let settings = settings.load_full();

        let shared_folder_settings = settings.shared_folder.as_ref().ok_or_else(|| {
            ApiError::bad_request().with_message("No shared folder configured for this server")
        })?;

        match shared_folder_settings {
            controller_settings::SharedFolder::Nextcloud {
                url,
                username,
                password,
                directory,
                expiry,
            } => {
                let client =
                    nextcloud_client::Client::new(url.clone(), username.clone(), password.clone())
                        .map_err(|e| {
                            warn!("Error creating NextCloud client: {e}");
                            ApiError::internal().with_message("Error creating NextCloud client")
                        })?;
                let path = format!(
                    "{}/opentalk-event-{}",
                    directory.trim_matches('/'),
                    event_id
                );
                let user_path = format!("files/{username}/{path}");
                client.create_folder(&user_path).await.map_err(|e| {
                    warn!("Error creating folder on NextCloud: {e}");
                    ApiError::internal().with_message("Error creating folder on NextCloud")
                })?;

                let expire_date = expiry
                    .as_ref()
                    .map(|days| Utc::now().date_naive() + Days::new(*days));

                let write_permissions = HashSet::from([
                    SharePermission::Read,
                    SharePermission::Create,
                    SharePermission::Update,
                    SharePermission::Delete,
                ]);
                let read_permissions = HashSet::from([SharePermission::Read]);

                async fn create_share(
                    client: &Client,
                    path: &str,
                    permissions: HashSet<SharePermission>,
                    label: &str,
                    password: String,
                    expire_date: Option<NaiveDate>,
                ) -> Result<(ShareId, SharedFolderAccess), ApiError> {
                    let mut creator = client
                        .create_share(path, ShareType::PublicLink)
                        .password(&password)
                        .label(label);
                    for permission in &permissions {
                        creator = creator.permission(*permission);
                    }
                    if let Some(expire_date) = expire_date {
                        creator = creator.expire_date(expire_date);
                    }
                    let share = creator.send().await.map_err(|e| {
                        warn!("Error creating share on NextCloud: {e}");
                        ApiError::internal().with_message("Error creating share on NextCloud")
                    })?;

                    // Workaround for NextCloud up to version 25 not processing the share permissions
                    // on folder creation. We just need to change them with a subsequent update request.
                    //
                    // See: https://github.com/nextcloud/server/issues/32611
                    if share.data.permissions != permissions {
                        client
                            .update_share(share.data.id.clone())
                            .permissions(permissions)
                            .await
                            .map_err(|e| {
                                warn!("Error setting permissions for share on NextCloud: {e}");
                                ApiError::internal().with_message(
                                    "Error setting permissions for share on NextCloud",
                                )
                            })?;
                    }

                    Ok((
                        share.data.id,
                        SharedFolderAccess {
                            url: share.data.url,
                            password,
                        },
                    ))
                }

                let write_password = generate_password(&client).await?;
                let read_password = generate_password(&client).await?;

                let (
                    write_share_id,
                    SharedFolderAccess {
                        url: write_url,
                        password: write_password,
                    },
                ) = create_share(
                    &client,
                    &path,
                    write_permissions,
                    "OpenTalk read-write",
                    write_password,
                    expire_date,
                )
                .await?;
                let (
                    read_share_id,
                    SharedFolderAccess {
                        url: read_url,
                        password: read_password,
                    },
                ) = create_share(
                    &client,
                    &path,
                    read_permissions,
                    "OpenTalk read-only",
                    read_password,
                    expire_date,
                )
                .await?;

                let new_shared_folder = NewEventSharedFolder {
                    event_id,
                    path,
                    write_share_id: write_share_id.to_string(),
                    write_url,
                    write_password,
                    read_share_id: read_share_id.to_string(),
                    read_url,
                    read_password,
                };

                let share = new_shared_folder
                    .try_insert(&mut conn)
                    .await?
                    .ok_or_else(ApiError::internal)?;

                Ok(Json(SharedFolder::from(share))
                    .customize()
                    .with_status(StatusCode::CREATED))
            }
        }
    }
}

pub async fn delete_shared_folders(
    settings: SharedSettingsActix,
    shared_folders: &[EventSharedFolder],
) -> Result<(), ApiError> {
    if shared_folders.is_empty() {
        return Ok(());
    }
    let settings = settings.load_full();

    let shared_folder_settings = if let Some(settings) = settings.shared_folder.as_ref() {
        settings
    } else {
        return Err(
            ApiError::bad_request().with_message("No shared folder configured for this server")
        );
    };

    match shared_folder_settings {
        controller_settings::SharedFolder::Nextcloud {
            url,
            username,
            password,
            ..
        } => {
            let client =
                nextcloud_client::Client::new(url.clone(), username.clone(), password.clone())
                    .map_err(|e| {
                        warn!("Error creating NextCloud client: {e}");
                        ApiError::internal().with_message("Error creating NextCloud client")
                    })?;
            for shared_folder in shared_folders {
                let path = &shared_folder.path;
                if path.trim_matches('/').is_empty() {
                    warn!("Preventing recursive deletion of empty shared folder path, this is probably harmful and not intended");
                    return Err(ApiError::internal());
                }
                let user_path = format!("files/{username}/{path}");
                if let Err(e) = client
                    .delete_share(ShareId::from(shared_folder.read_share_id.clone()))
                    .await
                {
                    warn!("Could not delete NextCloud read share: {e}");
                }
                if let Err(e) = client
                    .delete_share(ShareId::from(shared_folder.write_share_id.clone()))
                    .await
                {
                    warn!("Could not delete NextCloud write share: {e}");
                }
                match client.delete(&user_path).await {
                    Ok(()) | Err(nextcloud_client::Error::FileNotFound { .. }) => {}
                    Err(e) => {
                        warn!("Error deleting folder on NextCloud: {e}");
                        return Err(
                            ApiError::internal().with_message("Error deleting folder on NextCloud")
                        );
                    }
                };
            }
            Ok(())
        }
    }
}

#[delete("/events/{event_id}/shared_folder")]
pub async fn delete_shared_folder_for_event(
    settings: SharedSettingsActix,
    db: Data<Db>,
    event_id: Path<EventId>,
    query: Query<DeleteQuery>,
) -> Result<NoContent, ApiError> {
    let event_id = event_id.into_inner();

    let mut conn = db.get_conn().await?;

    let shared_folder = EventSharedFolder::get_for_event(&mut conn, event_id).await?;

    if let Some(shared_folder) = shared_folder {
        let shared_folders = std::slice::from_ref(&shared_folder);
        let deletion = delete_shared_folders(settings, shared_folders).await;
        match deletion {
            Ok(()) => {
                shared_folder.delete(&mut conn).await?;
                Ok(NoContent)
            }
            Err(e) => {
                if query.force_delete_reference_if_shared_folder_deletion_fails {
                    warn!(
                        "Deleting local shared folder reference anyway, because \
                        `force_delete_reference_if_shared_folder_deletion_fails` is set to true"
                    );
                    shared_folder.delete(&mut conn).await?;
                    Ok(NoContent)
                } else {
                    Err(e)
                }
            }
        }
    } else {
        Ok(NoContent)
    }
}

async fn generate_password(client: &Client) -> Result<String, ApiError> {
    client.generate_password().await.map_err(|e| {
        warn!("Error generating share password on NextCloud: {e}");
        ApiError::internal().with_message("Error generating share password NextCloud")
    })
}
