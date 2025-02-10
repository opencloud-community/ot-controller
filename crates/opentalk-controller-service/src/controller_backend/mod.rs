// SPDX-FileCopyrightText: OpenTalk GmbH <mail@opentalk.eu>
//
// SPDX-License-Identifier: EUPL-1.2

//! Provides the default [`OpenTalkControllerServiceBackend`] implementation.
mod assets;
mod auth;
mod rooms;

use std::{
    collections::{BTreeMap, BTreeSet},
    sync::Arc,
};

use async_trait::async_trait;
use bytes::Bytes;
use futures_core::Stream;
use kustos::Authz;
use opentalk_controller_service_facade::{OpenTalkControllerServiceBackend, RequestUser};
use opentalk_controller_settings::SharedSettings;
use opentalk_database::Db;
use opentalk_keycloak_admin::KeycloakAdminClient;
use opentalk_signaling_core::{
    assets::{ByStreamExt, NewAssetFileName},
    ExchangeHandle, ObjectStorage, ObjectStorageError,
};
use opentalk_types_api_v1::{
    assets::AssetResource,
    auth::{GetLoginResponseBody, OidcProvider},
    error::ApiError,
    pagination::PagePaginationQuery,
    rooms::{
        by_room_id::{assets::RoomsByRoomIdAssetsGetResponseBody, GetRoomEventResponseBody},
        GetRoomsResponseBody, RoomResource,
    },
};
use opentalk_types_common::{
    assets::AssetId,
    features::FeatureId,
    modules::ModuleId,
    rooms::{RoomId, RoomPassword},
    tariffs::TariffResource,
    users::UserId,
};

pub use crate::controller_backend::rooms::RoomsPoliciesBuilderExt;
use crate::services::MailService;

/// The default [`OpenTalkControllerServiceBackend`] implementation.
pub struct ControllerBackend {
    // TODO: these are ArcSwap in controller-core, investigate what exactly that provides and what it is used for
    settings: SharedSettings,
    authz: Authz,
    db: Arc<Db>,
    frontend_oidc_provider: OidcProvider,
    storage: Arc<ObjectStorage>,
    exchange_handle: ExchangeHandle,
    _mail_service: MailService,
    _kc_admin_client: Arc<KeycloakAdminClient>,
    module_features: BTreeMap<ModuleId, BTreeSet<FeatureId>>,
}

impl ControllerBackend {
    /// Create a new [`ControllerBackend`].
    #[allow(clippy::too_many_arguments)]
    pub fn new(
        settings: SharedSettings,
        authz: Authz,
        db: Arc<Db>,
        frontend_oidc_provider: OidcProvider,
        storage: Arc<ObjectStorage>,
        exchange_handle: ExchangeHandle,
        mail_service: MailService,
        kc_admin_client: Arc<KeycloakAdminClient>,
        module_features: BTreeMap<ModuleId, BTreeSet<FeatureId>>,
    ) -> Self {
        Self {
            settings,
            authz,
            db,
            frontend_oidc_provider,
            storage,
            exchange_handle,
            _mail_service: mail_service,
            _kc_admin_client: kc_admin_client,
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
        pagination: &PagePaginationQuery,
    ) -> Result<(GetRoomsResponseBody, i64), ApiError> {
        Ok(self.get_rooms(current_user_id, pagination).await?)
    }

    async fn create_room(
        &self,
        current_user: RequestUser,
        password: Option<RoomPassword>,
        enable_sip: bool,
        waiting_room: bool,
        e2e_encryption: bool,
    ) -> Result<RoomResource, ApiError> {
        Ok(self
            .create_room(
                current_user,
                password,
                enable_sip,
                waiting_room,
                e2e_encryption,
            )
            .await?)
    }

    async fn patch_room(
        &self,
        current_user: RequestUser,
        room_id: RoomId,
        password: Option<Option<RoomPassword>>,
        waiting_room: Option<bool>,
        e2e_encryption: Option<bool>,
    ) -> Result<RoomResource, ApiError> {
        Ok(self
            .patch_room(
                current_user,
                room_id,
                password,
                waiting_room,
                e2e_encryption,
            )
            .await?)
    }

    async fn delete_room(
        &self,
        current_user: RequestUser,
        room_id: RoomId,
        force_delete_reference_if_external_services_fail: bool,
    ) -> Result<(), ApiError> {
        Ok(self
            .delete_room(
                current_user,
                room_id,
                force_delete_reference_if_external_services_fail,
            )
            .await?)
    }

    async fn get_room(&self, room_id: &RoomId) -> Result<RoomResource, ApiError> {
        Ok(self.get_room(room_id).await?)
    }

    async fn get_room_tariff(&self, room_id: &RoomId) -> Result<TariffResource, ApiError> {
        Ok(self.get_room_tariff(room_id).await?)
    }

    async fn get_room_event(&self, room_id: &RoomId) -> Result<GetRoomEventResponseBody, ApiError> {
        Ok(self.get_room_event(room_id).await?)
    }

    async fn get_room_assets(
        &self,
        room_id: RoomId,
        pagination: PagePaginationQuery,
    ) -> Result<(RoomsByRoomIdAssetsGetResponseBody, i64), ApiError> {
        Ok(self.get_room_assets(room_id, pagination).await?)
    }

    async fn get_room_asset(
        &self,
        room_id: RoomId,
        asset_id: AssetId,
    ) -> Result<ByStreamExt, ApiError> {
        Ok(self.get_room_asset(room_id, asset_id).await?)
    }

    async fn create_room_asset(
        &self,
        room_id: RoomId,
        filename: NewAssetFileName,
        namespace: Option<ModuleId>,
        data: Box<dyn Stream<Item = Result<Bytes, ObjectStorageError>> + Unpin>,
    ) -> Result<AssetResource, ApiError> {
        Ok(self
            .create_room_asset(room_id, filename, namespace, data)
            .await?)
    }

    async fn delete_room_asset(&self, room_id: RoomId, asset_id: AssetId) -> Result<(), ApiError> {
        Ok(self.delete_room_asset(room_id, asset_id).await?)
    }
}
