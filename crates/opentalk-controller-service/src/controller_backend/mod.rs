// SPDX-FileCopyrightText: OpenTalk GmbH <mail@opentalk.eu>
//
// SPDX-License-Identifier: EUPL-1.2

mod auth;
mod rooms;

use std::{
    collections::{BTreeMap, BTreeSet},
    sync::Arc,
};

use async_trait::async_trait;
use opentalk_controller_service_facade::OpenTalkControllerServiceBackend;
use opentalk_controller_settings::SharedSettings;
use opentalk_database::Db;
use opentalk_types::api::error::ApiError;
use opentalk_types_api_v1::{
    auth::{GetLoginResponseBody, OidcProvider},
    rooms::{by_room_id::GetRoomEventResponseBody, RoomResource},
};
use opentalk_types_common::{
    features::FeatureId, modules::ModuleId, rooms::RoomId, tariffs::TariffResource,
};

/// The default [`OpenTalkControllerServiceBackend`] implementation.
pub struct ControllerBackend {
    db: Arc<Db>,
    // TODO: these are ArcSwap in controller-core, investigate what exactly that provides and what it is used for
    settings: SharedSettings,
    frontend_oidc_provider: OidcProvider,
    module_features: BTreeMap<ModuleId, BTreeSet<FeatureId>>,
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
            module_features: BTreeMap::default(),
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
    fn set_module_features(&mut self, module_features: BTreeMap<ModuleId, BTreeSet<FeatureId>>) {
        self.module_features = module_features;
    }

    async fn get_login(&self) -> GetLoginResponseBody {
        self.get_login().await
    }

    async fn get_room(&self, room_id: &RoomId) -> Result<RoomResource, ApiError> {
        self.get_room(room_id).await
    }

    async fn get_room_tariff(&self, room_id: &RoomId) -> Result<TariffResource, ApiError> {
        self.get_room_tariff(room_id).await
    }

    async fn get_room_event(&self, room_id: &RoomId) -> Result<GetRoomEventResponseBody, ApiError> {
        self.get_room_event(room_id).await
    }
}
