// SPDX-FileCopyrightText: OpenTalk GmbH <mail@opentalk.eu>
//
// SPDX-License-Identifier: EUPL-1.2

//! Provides the default [`OpenTalkControllerServiceBackend`] implementation.
mod auth;
mod rooms;

use std::{
    collections::{BTreeMap, BTreeSet},
    sync::Arc,
};

use async_trait::async_trait;
use kustos::Authz;
use opentalk_controller_service_facade::{OpenTalkControllerServiceBackend, RequestUser};
use opentalk_controller_settings::SharedSettings;
use opentalk_database::Db;
use opentalk_types::api::error::ApiError;
use opentalk_types_api_v1::{
    auth::{GetLoginResponseBody, OidcProvider},
    rooms::{by_room_id::GetRoomEventResponseBody, GetRoomsResponseBody, RoomResource},
};
use opentalk_types_common::{
    features::FeatureId,
    modules::ModuleId,
    rooms::{RoomId, RoomPassword},
    tariffs::TariffResource,
    users::UserId,
};

pub use crate::controller_backend::rooms::RoomsPoliciesBuilderExt;

/// The default [`OpenTalkControllerServiceBackend`] implementation.
pub struct ControllerBackend {
    // TODO: these are ArcSwap in controller-core, investigate what exactly that provides and what it is used for
    settings: SharedSettings,
    authz: Authz,
    db: Arc<Db>,
    frontend_oidc_provider: OidcProvider,
    module_features: BTreeMap<ModuleId, BTreeSet<FeatureId>>,
}

impl ControllerBackend {
    /// Create a new [`ControllerBackend`].
    pub fn new(
        settings: SharedSettings,
        authz: Authz,
        db: Arc<Db>,
        frontend_oidc_provider: OidcProvider,
        module_features: BTreeMap<ModuleId, BTreeSet<FeatureId>>,
    ) -> Self {
        Self {
            settings,
            authz,
            db,
            frontend_oidc_provider,
            module_features,
        }
    }
}

impl std::fmt::Debug for ControllerBackend {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        write!(f, "ControllerBackend")
    }
}

#[async_trait(?Send)]
impl OpenTalkControllerServiceBackend for ControllerBackend {
    async fn get_login(&self) -> GetLoginResponseBody {
        self.get_login().await
    }

    async fn get_rooms(
        &self,
        current_user_id: UserId,
        per_page: i64,
        page: i64,
    ) -> Result<(GetRoomsResponseBody, i64), ApiError> {
        self.get_rooms(current_user_id, per_page, page).await
    }

    async fn create_room(
        &self,
        current_user: RequestUser,
        password: Option<RoomPassword>,
        enable_sip: bool,
        waiting_room: bool,
        e2e_encryption: bool,
    ) -> Result<RoomResource, ApiError> {
        self.create_room(
            current_user,
            password,
            enable_sip,
            waiting_room,
            e2e_encryption,
        )
        .await
    }

    async fn patch_room(
        &self,
        current_user: RequestUser,
        room_id: RoomId,
        password: Option<Option<RoomPassword>>,
        waiting_room: Option<bool>,
        e2e_encryption: Option<bool>,
    ) -> Result<RoomResource, ApiError> {
        self.patch_room(
            current_user,
            room_id,
            password,
            waiting_room,
            e2e_encryption,
        )
        .await
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
