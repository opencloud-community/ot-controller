// SPDX-FileCopyrightText: OpenTalk GmbH <mail@opentalk.eu>
//
// SPDX-License-Identifier: EUPL-1.2

use std::sync::Arc;

use opentalk_types::api::error::ApiError;
use opentalk_types_api_v1::{
    auth::GetLoginResponseBody,
    rooms::{by_room_id::GetRoomEventResponseBody, RoomResource},
};
use opentalk_types_common::rooms::RoomId;
use tokio::sync::RwLock;

use crate::OpenTalkControllerServiceBackend;

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

    /// Get a room
    pub async fn get_room(&self, room_id: &RoomId) -> Result<RoomResource, ApiError> {
        self.backend.read().await.get_room(room_id).await
    }

    /// Get a room's event
    pub async fn get_room_event(
        &self,
        room_id: &RoomId,
    ) -> Result<GetRoomEventResponseBody, ApiError> {
        self.backend.read().await.get_room_event(room_id).await
    }
}
