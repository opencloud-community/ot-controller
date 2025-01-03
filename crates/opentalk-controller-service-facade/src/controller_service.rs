// SPDX-FileCopyrightText: OpenTalk GmbH <mail@opentalk.eu>
//
// SPDX-License-Identifier: EUPL-1.2

use std::sync::Arc;

use opentalk_types::api::error::ApiError;
use opentalk_types_api_v1::{
    auth::GetLoginResponseBody,
    rooms::{by_room_id::GetRoomEventResponseBody, GetRoomsResponseBody, RoomResource},
};
use opentalk_types_common::{
    rooms::{RoomId, RoomPassword},
    tariffs::TariffResource,
    users::UserId,
};
use tokio::sync::RwLock;

use crate::{OpenTalkControllerServiceBackend, RequestUser};

/// Thread-safe handle to a [`OpenTalkControllerServiceBackend`] implementation.
#[derive(Clone)]
pub struct OpenTalkControllerService {
    backend: Arc<RwLock<dyn OpenTalkControllerServiceBackend>>,
}

impl std::fmt::Debug for OpenTalkControllerService {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        write!(f, "OpenTalkControllerService")
    }
}

impl OpenTalkControllerService {
    /// Create a new [`OpenTalkControllerService`] holding a type that implements [`OpenTalkControllerServiceBackend`].
    pub fn new<B: OpenTalkControllerServiceBackend + 'static>(backend: B) -> Self {
        Self {
            backend: Arc::new(RwLock::new(backend)),
        }
    }

    /// Get the configured OIDC provider
    pub async fn get_login(&self) -> GetLoginResponseBody {
        self.backend.read().await.get_login().await
    }

    /// Get all accessible rooms
    pub async fn get_rooms(
        &self,
        current_user_id: UserId,
        per_page: i64,
        page: i64,
    ) -> Result<(GetRoomsResponseBody, i64), ApiError> {
        self.backend
            .read()
            .await
            .get_rooms(current_user_id, per_page, page)
            .await
    }

    /// Create a new room
    pub async fn create_room(
        &self,
        password: Option<RoomPassword>,
        enable_sip: bool,
        waiting_room: bool,
        e2e_encryption: bool,
        current_user: RequestUser,
    ) -> Result<RoomResource, ApiError> {
        self.backend
            .read()
            .await
            .create_room(
                password,
                enable_sip,
                waiting_room,
                e2e_encryption,
                current_user,
            )
            .await
    }

    /// Get a room
    pub async fn get_room(&self, room_id: &RoomId) -> Result<RoomResource, ApiError> {
        self.backend.read().await.get_room(room_id).await
    }

    /// Get a room's tariff
    pub async fn get_room_tariff(&self, room_id: &RoomId) -> Result<TariffResource, ApiError> {
        self.backend.read().await.get_room_tariff(room_id).await
    }

    /// Get a room's event
    pub async fn get_room_event(
        &self,
        room_id: &RoomId,
    ) -> Result<GetRoomEventResponseBody, ApiError> {
        self.backend.read().await.get_room_event(room_id).await
    }
}
