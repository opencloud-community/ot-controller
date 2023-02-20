// SPDX-FileCopyrightText: OpenTalk GmbH <mail@opentalk.eu>
//
// SPDX-License-Identifier: EUPL-1.2

use crate::api::internal::NoContent;
use crate::api::signaling::control;
use crate::api::v1::events::associated_resource_ids;
use crate::api::v1::events::shared_folder::delete_shared_folders;
use crate::api::v1::response::ApiError;
use crate::exchange_task::ExchangeHandle;
use crate::settings::SharedSettingsActix;
use crate::storage::assets::asset_key;
use crate::storage::ObjectStorage;
use actix_web::delete;
use actix_web::web::{Data, Path, ReqData};
use database::{DatabaseError, Db};
use db_storage::assets::Asset;
use db_storage::events::shared_folders::EventSharedFolder;
use db_storage::events::Event;
use db_storage::module_resources::ModuleResource;
use db_storage::rooms::Room;
use db_storage::sip_configs::SipConfig;
use db_storage::users::User;
use diesel_async::scoped_futures::ScopedFutureExt;
use diesel_async::AsyncConnection;
use kustos::prelude::*;
use types::core::RoomId;

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
pub async fn delete(
    settings: SharedSettingsActix,
    db: Data<Db>,
    storage: Data<ObjectStorage>,
    exchange_handle: Data<ExchangeHandle>,
    room_id: Path<RoomId>,
    current_user: ReqData<User>,
    authz: Data<Authz>,
) -> Result<NoContent, ApiError> {
    let room_id = room_id.into_inner();
    let current_user = current_user.into_inner();
    let room_path = format!("/rooms/{room_id}");

    let mut conn = db.get_conn().await?;

    let mut linked_events = Event::get_all_ids_for_room(&mut conn, room_id).await?;
    let mut linked_module_resources =
        ModuleResource::get_all_ids_for_room(&mut conn, room_id).await?;
    let mut linked_shared_folders = EventSharedFolder::get_all_for_room(&mut conn, room_id).await?;

    // Sort for improved equality comparison later on, inside the transaction.
    linked_events.sort();
    linked_module_resources.sort();
    linked_shared_folders.sort_by(|a, b| a.event_id.cmp(&b.event_id));

    // Enforce access to all DELETE operations
    let mut resources = linked_events
        .iter()
        .map(|e| e.resource_id())
        .chain(linked_module_resources.iter().map(|e| e.resource_id()))
        .chain(
            linked_shared_folders
                .iter()
                .map(|f| f.event_id.resource_id().with_suffix("/shared_folder")),
        )
        .collect::<Vec<_>>();

    resources.push(room_path.clone().into());

    let checked = authz
        .check_batched(current_user.id, resources.clone(), AccessMethod::DELETE)
        .await?;

    if checked.iter().any(|&res| !res) {
        return Err(ApiError::forbidden());
    }

    let message = types::signaling::NamespacedEvent {
        namespace: control::NAMESPACE,
        timestamp: types::core::Timestamp::now(),
        payload: control::exchange::Message::RoomDeleted,
    };

    if let Err(e) = exchange_handle.publish(
        control::exchange::global_room_all_participants(room_id),
        serde_json::to_string(&message).expect("Failed to convert namespaced to json"),
    ) {
        log::warn!("Failed to publish message to exchange, {}", e);
    }

    delete_shared_folders(settings, &linked_shared_folders).await?;

    let resources: Vec<_> = linked_events
        .iter()
        .flat_map(|&event_id| associated_resource_ids(event_id))
        .chain(linked_module_resources.iter().map(|e| e.resource_id()))
        .chain(
            linked_shared_folders
                .iter()
                .map(|f| f.event_id.resource_id().with_suffix("/shared_folder")),
        )
        .chain(associated_room_resource_ids(room_id))
        .collect();

    let assets = conn
        .transaction(|conn| {
            async move {
                // We check if in the meantime (during the permission check) another event got linked to
                let mut current_events = Event::get_all_ids_for_room(conn, room_id).await?;
                current_events.sort();

                if current_events != linked_events {
                    return Err(DatabaseError::custom("Race-condition during access checks"));
                }

                let mut current_module_resources =
                    ModuleResource::get_all_ids_for_room(conn, room_id).await?;
                current_module_resources.sort();

                if current_module_resources != linked_module_resources {
                    return Err(DatabaseError::custom("Race-condition during access checks"));
                }

                let mut current_shared_folders =
                    EventSharedFolder::get_all_for_room(conn, room_id).await?;
                current_shared_folders.sort_by(|a, b| a.event_id.cmp(&b.event_id));
                if current_shared_folders != linked_shared_folders {
                    return Err(DatabaseError::custom("Race-condition during access checks"));
                }

                let shared_folder_event_ids = current_shared_folders
                    .into_iter()
                    .map(|e| e.event_id)
                    .collect::<Vec<_>>();

                let mut current_assets = Asset::get_all_ids_for_room(conn, room_id).await?;
                current_assets.sort();

                EventSharedFolder::delete_by_event_ids(conn, &shared_folder_event_ids).await?;
                ModuleResource::delete_by_room(conn, room_id).await?;
                Event::delete_all_for_room(conn, room_id).await?;
                SipConfig::delete_by_room(conn, room_id).await?;
                Asset::delete_by_ids(conn, &current_assets).await?;
                Room::delete_by_id(conn, room_id).await?;

                Ok(current_assets)
            }
            .scope_boxed()
        })
        .await?;

    drop(conn);

    for asset_id in assets {
        storage.delete(asset_key(&asset_id)).await?;
    }

    authz.remove_explicit_resources(resources).await?;

    Ok(NoContent {})
}

pub(crate) fn associated_room_resource_ids(
    room_id: RoomId,
) -> impl IntoIterator<Item = ResourceId> {
    [
        ResourceId::from(format!("/rooms/{room_id}")),
        ResourceId::from(format!("/rooms/{room_id}/invites")),
        ResourceId::from(format!("/rooms/{room_id}/invites/*")),
        ResourceId::from(format!("/rooms/{room_id}/start")),
    ]
}
