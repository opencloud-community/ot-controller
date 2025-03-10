// SPDX-FileCopyrightText: OpenTalk GmbH <mail@opentalk.eu>
//
// SPDX-License-Identifier: EUPL-1.2

//! Provides the default [`OpenTalkControllerServiceBackend`] implementation.
mod assets;
mod auth;
mod events;
mod invites;
mod rooms;
mod sip_configs;
mod streaming_targets;
mod users;

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
    assets::{AssetResource, AssetSortingQuery},
    auth::{GetLoginResponseBody, OidcProvider},
    error::ApiError,
    events::StreamingTargetOptionsQuery,
    pagination::PagePaginationQuery,
    rooms::{
        by_room_id::{
            assets::RoomsByRoomIdAssetsGetResponseBody,
            invites::{
                GetRoomsInvitesResponseBody, InviteResource, PostInviteRequestBody,
                PostInviteVerifyRequestBody, PostInviteVerifyResponseBody, PutInviteRequestBody,
            },
            sip::{PutSipConfigRequestBody, SipConfigResource},
            streaming_targets::{
                GetRoomStreamingTargetResponseBody, GetRoomStreamingTargetsResponseBody,
                PatchRoomStreamingTargetRequestBody, PatchRoomStreamingTargetResponseBody,
                PostRoomStreamingTargetResponseBody, RoomAndStreamingTargetId,
            },
            GetRoomEventResponseBody,
        },
        GetRoomsResponseBody, RoomResource,
    },
    users::{
        me::PatchMeRequestBody, GetFindQuery, GetFindResponseBody, GetUserAssetsResponseBody,
        PrivateUserProfile, PublicUserProfile,
    },
};
use opentalk_types_common::{
    assets::AssetId,
    events::EventId,
    features::FeatureId,
    modules::ModuleId,
    rooms::{invite_codes::InviteCode, RoomId, RoomPassword},
    streaming::StreamingTarget,
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
    mail_service: MailService,
    kc_admin_client: Arc<KeycloakAdminClient>,
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
            mail_service,
            kc_admin_client,
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
        pagination: &PagePaginationQuery,
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

    async fn create_invite(
        &self,
        current_user: RequestUser,
        room_id: RoomId,
        new_invite: PostInviteRequestBody,
    ) -> Result<InviteResource, ApiError> {
        Ok(self
            .create_invite(current_user, room_id, new_invite)
            .await?)
    }

    async fn get_invites(
        &self,
        room_id: RoomId,
        pagination: &PagePaginationQuery,
    ) -> Result<(GetRoomsInvitesResponseBody, i64), ApiError> {
        Ok(self.get_invites(room_id, pagination).await?)
    }

    async fn get_invite(
        &self,
        room_id: RoomId,
        invite_code: InviteCode,
    ) -> Result<InviteResource, ApiError> {
        Ok(self.get_invite(room_id, invite_code).await?)
    }

    async fn update_invite(
        &self,
        current_user: RequestUser,
        room_id: RoomId,
        invite_code: InviteCode,
        body: PutInviteRequestBody,
    ) -> Result<InviteResource, ApiError> {
        Ok(self
            .update_invite(current_user, room_id, invite_code, body)
            .await?)
    }

    async fn delete_invite(
        &self,
        current_user: RequestUser,
        room_id: RoomId,
        invite_code: InviteCode,
    ) -> Result<(), ApiError> {
        Ok(self
            .delete_invite(current_user, room_id, invite_code)
            .await?)
    }

    async fn verify_invite_code(
        &self,
        data: PostInviteVerifyRequestBody,
    ) -> Result<PostInviteVerifyResponseBody, ApiError> {
        Ok(self.verify_invite_code(data).await?)
    }

    async fn get_sip_config(&self, room_id: RoomId) -> Result<SipConfigResource, ApiError> {
        Ok(self.get_sip_config(room_id).await?)
    }

    async fn set_sip_config(
        &self,
        room_id: RoomId,
        modify_sip_config: PutSipConfigRequestBody,
    ) -> Result<(SipConfigResource, bool), ApiError> {
        Ok(self.set_sip_config(room_id, modify_sip_config).await?)
    }

    async fn delete_sip_config(&self, room_id: RoomId) -> Result<(), ApiError> {
        Ok(self.delete_sip_config(room_id).await?)
    }

    async fn get_streaming_targets(
        &self,
        user_id: UserId,
        room_id: RoomId,
        pagination: &PagePaginationQuery,
    ) -> Result<GetRoomStreamingTargetsResponseBody, ApiError> {
        Ok(self
            .get_streaming_targets(user_id, room_id, pagination)
            .await?)
    }

    async fn post_streaming_target(
        &self,
        current_user: RequestUser,
        room_id: RoomId,
        query: StreamingTargetOptionsQuery,
        streaming_target: StreamingTarget,
    ) -> Result<PostRoomStreamingTargetResponseBody, ApiError> {
        Ok(self
            .post_streaming_target(current_user, room_id, query, streaming_target)
            .await?)
    }

    async fn get_streaming_target(
        &self,
        user_id: UserId,
        path_params: RoomAndStreamingTargetId,
    ) -> Result<GetRoomStreamingTargetResponseBody, ApiError> {
        Ok(self.get_streaming_target(user_id, path_params).await?)
    }

    async fn patch_streaming_target(
        &self,
        current_user: RequestUser,
        path_params: RoomAndStreamingTargetId,
        query: StreamingTargetOptionsQuery,
        streaming_target: PatchRoomStreamingTargetRequestBody,
    ) -> Result<PatchRoomStreamingTargetResponseBody, ApiError> {
        Ok(self
            .patch_streaming_target(current_user, path_params, query, streaming_target)
            .await?)
    }

    async fn delete_streaming_target(
        &self,
        current_user: RequestUser,
        path_params: RoomAndStreamingTargetId,
        query: StreamingTargetOptionsQuery,
    ) -> Result<(), ApiError> {
        Ok(self
            .delete_streaming_target(current_user, path_params, query)
            .await?)
    }

    async fn add_event_to_favorites(
        &self,
        current_user: RequestUser,
        event_id: EventId,
    ) -> Result<bool, ApiError> {
        Ok(self.add_event_to_favorites(current_user, event_id).await?)
    }

    async fn remove_event_from_favorites(
        &self,
        current_user: RequestUser,
        event_id: EventId,
    ) -> Result<(), ApiError> {
        Ok(self
            .remove_event_from_favorites(current_user, event_id)
            .await?)
    }

    async fn patch_me(
        &self,
        current_user: RequestUser,
        patch: PatchMeRequestBody,
    ) -> Result<Option<PrivateUserProfile>, ApiError> {
        Ok(self.patch_me(current_user, patch).await?)
    }

    async fn get_me(&self, current_user: RequestUser) -> Result<PrivateUserProfile, ApiError> {
        Ok(self.get_me(current_user).await?)
    }

    async fn get_my_tariff(&self, current_user: RequestUser) -> Result<TariffResource, ApiError> {
        Ok(self.get_my_tariff(current_user).await?)
    }

    async fn get_my_assets(
        &self,
        current_user: RequestUser,
        sorting: AssetSortingQuery,
        pagination: &PagePaginationQuery,
    ) -> Result<(GetUserAssetsResponseBody, i64), ApiError> {
        Ok(self
            .get_my_assets(current_user, sorting, pagination)
            .await?)
    }

    async fn get_user(
        &self,
        current_user: RequestUser,
        user_id: UserId,
    ) -> Result<PublicUserProfile, ApiError> {
        Ok(self.get_user(current_user, user_id).await?)
    }

    async fn find_users(
        &self,
        current_user: RequestUser,
        query: GetFindQuery,
    ) -> Result<GetFindResponseBody, ApiError> {
        Ok(self.find_users(current_user, query).await?)
    }
}
