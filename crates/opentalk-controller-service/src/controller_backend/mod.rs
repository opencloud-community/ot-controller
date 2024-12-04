// SPDX-FileCopyrightText: OpenTalk GmbH <mail@opentalk.eu>
//
// SPDX-License-Identifier: EUPL-1.2

mod auth;
mod rooms;

use std::sync::Arc;

use async_trait::async_trait;
use opentalk_controller_service_facade::OpenTalkControllerServiceBackend;
use opentalk_controller_settings::SharedSettings;
use opentalk_database::Db;
use opentalk_types::api::error::ApiError;
use opentalk_types_api_v1::{
    auth::{GetLoginResponseBody, OidcProvider},
    rooms::{by_room_id::GetRoomEventResponseBody, RoomResource},
};
use opentalk_types_common::rooms::RoomId;

/// The default [`OpenTalkControllerServiceBackend`] implementation.
pub struct ControllerBackend {
    db: Arc<Db>,
    // TODO: these are ArcSwap in controller-core, investigate what exactly that provides and what it is used for
    settings: SharedSettings,
    frontend_oidc_provider: OidcProvider,
}

impl ControllerBackend {
    /// Create a new [`ControllerBackend`].
    pub fn new(
        settings: SharedSettings,
        db: Arc<Db>,
        frontend_oidc_provider: OidcProvider,
    ) -> Self {
        Self {
            settings,
            db,
            frontend_oidc_provider,
        }
    }
}

impl std::fmt::Debug for ControllerBackend {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        write!(f, "ControllerBackend")
    }
}

#[async_trait]
impl OpenTalkControllerServiceBackend for ControllerBackend {
    async fn get_login(&self) -> GetLoginResponseBody {
        self.get_login().await
    }

    async fn get_room(&self, room_id: &RoomId) -> Result<RoomResource, ApiError> {
        self.get_room(room_id).await
    }

    async fn get_room_event(&self, room_id: &RoomId) -> Result<GetRoomEventResponseBody, ApiError> {
        self.get_room_event(room_id).await
    }
}
