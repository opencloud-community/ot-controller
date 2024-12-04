// SPDX-FileCopyrightText: OpenTalk GmbH <mail@opentalk.eu>
//
// SPDX-License-Identifier: EUPL-1.2

use std::collections::{BTreeMap, BTreeSet};

use async_trait::async_trait;
use opentalk_types::api::error::ApiError;
use opentalk_types_api_v1::{
    auth::GetLoginResponseBody,
    rooms::{by_room_id::GetRoomEventResponseBody, RoomResource},
};
use opentalk_types_common::{
    features::FeatureId, modules::ModuleId, rooms::RoomId, tariffs::TariffResource,
};

/// Trait implemented by OpenTalk controller service backends
#[async_trait]
pub trait OpenTalkControllerServiceBackend: Send + Sync {
    /// Set the available modules and their features
    fn set_module_features(&mut self, module_features: BTreeMap<ModuleId, BTreeSet<FeatureId>>);

    /// Get the configured OIDC provider
    async fn get_login(&self) -> GetLoginResponseBody;

    /// Get a room
    async fn get_room(&self, room_id: &RoomId) -> Result<RoomResource, ApiError>;

    /// Get a room's tariff
    async fn get_room_tariff(&self, room_id: &RoomId) -> Result<TariffResource, ApiError>;

    /// Get a room's event
    async fn get_room_event(&self, room_id: &RoomId) -> Result<GetRoomEventResponseBody, ApiError>;
}
