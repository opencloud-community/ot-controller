// SPDX-FileCopyrightText: OpenTalk GmbH <mail@opentalk.eu>
//
// SPDX-License-Identifier: EUPL-1.2

use opentalk_db_storage::events::shared_folders::{EventSharedFolder, NewEventSharedFolder};
use opentalk_inventory::{error::StorageBackendSnafu, EventSharedFolderInventory};
use opentalk_types_common::{events::EventId, rooms::RoomId};
use snafu::ResultExt as _;

use crate::{DatabaseConnection, Result};

#[async_trait::async_trait]
impl EventSharedFolderInventory for DatabaseConnection {
    #[tracing::instrument(err, skip_all)]
    async fn try_create_event_shared_folder(
        &mut self,
        new_shared_folder: NewEventSharedFolder,
    ) -> Result<Option<EventSharedFolder>> {
        new_shared_folder
            .try_insert(&mut self.inner)
            .await
            .context(StorageBackendSnafu)
    }

    #[tracing::instrument(err, skip_all)]
    async fn get_event_shared_folder(
        &mut self,
        event_id: EventId,
    ) -> Result<Option<EventSharedFolder>> {
        EventSharedFolder::get_for_event(&mut self.inner, event_id)
            .await
            .context(StorageBackendSnafu)
    }

    #[tracing::instrument(err, skip_all)]
    async fn get_event_shared_folders_for_room(
        &mut self,
        room_id: RoomId,
    ) -> Result<Vec<EventSharedFolder>> {
        EventSharedFolder::get_all_for_room(&mut self.inner, room_id)
            .await
            .context(StorageBackendSnafu)
    }

    #[tracing::instrument(err, skip_all)]
    async fn delete_shared_folder_by_event_id(&mut self, event_id: EventId) -> Result<()> {
        EventSharedFolder::delete_by_event_id(&mut self.inner, event_id)
            .await
            .context(StorageBackendSnafu)
    }

    #[tracing::instrument(err, skip_all)]
    async fn delete_shared_folders_by_event_ids(&mut self, event_ids: &[EventId]) -> Result<()> {
        EventSharedFolder::delete_by_event_ids(&mut self.inner, event_ids)
            .await
            .context(StorageBackendSnafu)
    }
}
