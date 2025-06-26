// SPDX-FileCopyrightText: OpenTalk GmbH <mail@opentalk.eu>
//
// SPDX-License-Identifier: EUPL-1.2

use opentalk_db_storage::{
    rooms::{NewRoom, Room, UpdateRoom},
    users::User,
};
use opentalk_inventory::{error::StorageBackendSnafu, RoomInventory};
use opentalk_types_common::rooms::RoomId;
use snafu::ResultExt as _;

use crate::{DatabaseConnection, Result};

#[async_trait::async_trait]
impl RoomInventory for DatabaseConnection {
    #[tracing::instrument(err, skip_all)]
    async fn create_room(&mut self, new_room: NewRoom) -> Result<Room> {
        new_room
            .insert(&mut self.inner)
            .await
            .context(StorageBackendSnafu)
    }

    #[tracing::instrument(err, skip_all)]
    async fn get_room(&mut self, room_id: RoomId) -> Result<Room> {
        Room::get(&mut self.inner, room_id)
            .await
            .context(StorageBackendSnafu)
    }

    #[tracing::instrument(err, skip_all)]
    async fn get_room_with_creator(&mut self, room_id: RoomId) -> Result<(Room, User)> {
        Room::get_with_user(&mut self.inner, room_id)
            .await
            .context(StorageBackendSnafu)
    }

    #[tracing::instrument(err, skip_all)]
    async fn update_room(&mut self, room_id: RoomId, update: UpdateRoom) -> Result<Room> {
        update
            .apply(&mut self.inner, room_id)
            .await
            .context(StorageBackendSnafu)
    }

    #[tracing::instrument(err, skip_all)]
    async fn delete_room(&mut self, room_id: RoomId) -> Result<()> {
        Room::delete_by_id(&mut self.inner, room_id)
            .await
            .context(StorageBackendSnafu)
    }

    #[tracing::instrument(err, skip_all)]
    async fn get_all_orphaned_room_ids(&mut self) -> Result<Vec<RoomId>> {
        Room::get_all_orphaned_ids(&mut self.inner)
            .await
            .context(StorageBackendSnafu)
    }

    #[tracing::instrument(err, skip_all)]
    async fn get_all_rooms_paginated_with_creator(
        &mut self,
        limit: i64,
        page: i64,
    ) -> Result<(Vec<(Room, User)>, i64)> {
        Room::get_all_with_creator_paginated(&mut self.inner, limit, page)
            .await
            .context(StorageBackendSnafu)
    }

    #[tracing::instrument(err, skip_all)]
    async fn get_rooms_paginated_by_id_with_creator(
        &mut self,
        room_ids: &[RoomId],
        limit: i64,
        page: i64,
    ) -> Result<(Vec<(Room, User)>, i64)> {
        Room::get_by_ids_with_creator_paginated(&mut self.inner, room_ids, limit, page)
            .await
            .context(StorageBackendSnafu)
    }
}
