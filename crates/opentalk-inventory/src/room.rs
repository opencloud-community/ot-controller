// SPDX-FileCopyrightText: OpenTalk GmbH <mail@opentalk.eu>
//
// SPDX-License-Identifier: EUPL-1.2

use opentalk_db_storage::{
    rooms::{NewRoom, Room, UpdateRoom},
    users::User,
};
use opentalk_types_common::rooms::RoomId;

use crate::Result;

/// A trait for retrieving and storing room entities.
#[async_trait::async_trait]
pub trait RoomInventory {
    /// Create a new room.
    async fn create_room(&mut self, new_room: NewRoom) -> Result<Room>;

    /// Get a room by its id.
    async fn get_room(&mut self, room_id: RoomId) -> Result<Room>;

    /// Get a room and its creator by its id.
    async fn get_room_with_creator(&mut self, room_id: RoomId) -> Result<(Room, User)>;

    /// Update a room.
    async fn update_room(&mut self, room_id: RoomId, update: UpdateRoom) -> Result<Room>;

    /// Delete a room.
    async fn delete_room(&mut self, room_id: RoomId) -> Result<()>;

    /// Get all rooms that don't have an event associated.
    async fn get_all_orphaned_room_ids(&mut self) -> Result<Vec<RoomId>>;

    /// Get all rooms, paginated and with the creator user.
    ///
    /// The returned tuple contains a `Vec` with the data, and the total number of available rooms.
    async fn get_all_rooms_paginated_with_creator(
        &mut self,
        limit: i64,
        page: i64,
    ) -> Result<(Vec<(Room, User)>, i64)>;

    /// Get a set of rooms by their id, paginated and with the creator user.
    ///
    /// The returned tuple contains a `Vec` with the data, and the total number of available rooms.
    async fn get_rooms_paginated_by_id_with_creator(
        &mut self,
        room_ids: &[RoomId],
        limit: i64,
        page: i64,
    ) -> Result<(Vec<(Room, User)>, i64)>;
}
