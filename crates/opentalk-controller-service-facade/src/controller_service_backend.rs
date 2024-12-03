// SPDX-FileCopyrightText: OpenTalk GmbH <mail@opentalk.eu>
//
// SPDX-License-Identifier: EUPL-1.2

use async_trait::async_trait;
use opentalk_types::api::error::ApiError;
use opentalk_types_api_v1::{
    auth::GetLoginResponseBody,
    rooms::{by_room_id::GetRoomEventResponseBody, RoomResource},
};
use opentalk_types_common::rooms::RoomId;

/// Trait implemented by OpenTalk controller service backends
#[async_trait]
pub trait OpenTalkControllerServiceBackend: Send + Sync {
    /// Get the configured OIDC provider
    async fn get_login(&self) -> GetLoginResponseBody;

    /// Get a room
    async fn get_room(&self, room_id: &RoomId) -> Result<RoomResource, ApiError>;

    /// Get a room's event
    async fn get_room_event(&self, room_id: &RoomId) -> Result<GetRoomEventResponseBody, ApiError>;
}
