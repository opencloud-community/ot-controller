// SPDX-FileCopyrightText: OpenTalk GmbH <mail@opentalk.eu>
//
// SPDX-License-Identifier: EUPL-1.2

use async_trait::async_trait;
use opentalk_types_api_v1::{
    auth::GetLoginResponseBody,
    error::ApiError,
    rooms::{by_room_id::GetRoomEventResponseBody, GetRoomsResponseBody, RoomResource},
};
use opentalk_types_common::{
    rooms::{RoomId, RoomPassword},
    tariffs::TariffResource,
    users::UserId,
};

use crate::RequestUser;

/// Trait implemented by OpenTalk controller service backends
#[async_trait(?Send)]
pub trait OpenTalkControllerServiceBackend: Send + Sync {
    /// Get the configured OIDC provider
    async fn get_login(&self) -> GetLoginResponseBody;

    /// Get all accessible rooms
    async fn get_rooms(
        &self,
        current_user_id: UserId,
        per_page: i64,
        page: i64,
    ) -> Result<(GetRoomsResponseBody, i64), ApiError>;

    /// Create a new room
    async fn create_room(
        &self,
        current_user: RequestUser,
        password: Option<RoomPassword>,
        enable_sip: bool,
        waiting_room: bool,
        e2e_encryption: bool,
    ) -> Result<RoomResource, ApiError>;

    /// Patch a room with the provided fields
    async fn patch_room(
        &self,
        current_user: RequestUser,
        room_id: RoomId,
        password: Option<Option<RoomPassword>>,
        waiting_room: Option<bool>,
        e2e_encryption: Option<bool>,
    ) -> Result<RoomResource, ApiError>;

    /// Get a room
    async fn get_room(&self, room_id: &RoomId) -> Result<RoomResource, ApiError>;

    /// Get a room's tariff
    async fn get_room_tariff(&self, room_id: &RoomId) -> Result<TariffResource, ApiError>;

    /// Get a room's event
    async fn get_room_event(&self, room_id: &RoomId) -> Result<GetRoomEventResponseBody, ApiError>;
}
