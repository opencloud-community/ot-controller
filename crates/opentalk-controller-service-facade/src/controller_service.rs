// SPDX-FileCopyrightText: OpenTalk GmbH <mail@opentalk.eu>
//
// SPDX-License-Identifier: EUPL-1.2

use std::sync::Arc;

use bytes::Bytes;
use futures_core::Stream;
use opentalk_signaling_core::{
    assets::{ByStreamExt, NewAssetFileName},
    ObjectStorageError,
};
use opentalk_types_api_v1::{
    assets::AssetResource,
    auth::GetLoginResponseBody,
    error::ApiError,
    pagination::PagePaginationQuery,
    rooms::{
        by_room_id::{
            assets::RoomsByRoomIdAssetsGetResponseBody,
            invites::{
                GetRoomsInvitesResponseBody, InviteResource, PostInviteRequestBody,
                PostInviteVerifyRequestBody, PostInviteVerifyResponseBody, PutInviteRequestBody,
            },
            sip::{PutSipConfigRequestBody, SipConfigResource},
            GetRoomEventResponseBody,
        },
        GetRoomsResponseBody, RoomResource,
    },
};
use opentalk_types_common::{
    assets::AssetId,
    events::EventId,
    modules::ModuleId,
    rooms::{invite_codes::InviteCode, RoomId, RoomPassword},
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
        pagination: &PagePaginationQuery,
    ) -> Result<(GetRoomsResponseBody, i64), ApiError> {
        self.backend
            .read()
            .await
            .get_rooms(current_user_id, pagination)
            .await
    }

    /// Create a new room
    pub async fn create_room(
        &self,
        current_user: RequestUser,
        password: Option<RoomPassword>,
        enable_sip: bool,
        waiting_room: bool,
        e2e_encryption: bool,
    ) -> Result<RoomResource, ApiError> {
        self.backend
            .read()
            .await
            .create_room(
                current_user,
                password,
                enable_sip,
                waiting_room,
                e2e_encryption,
            )
            .await
    }

    /// Patch a room with the provided fields
    pub async fn patch_room(
        &self,
        current_user: RequestUser,
        room_id: RoomId,
        password: Option<Option<RoomPassword>>,
        waiting_room: Option<bool>,
        e2e_encryption: Option<bool>,
    ) -> Result<RoomResource, ApiError> {
        self.backend
            .read()
            .await
            .patch_room(
                current_user,
                room_id,
                password,
                waiting_room,
                e2e_encryption,
            )
            .await
    }

    /// Delete a room and its owned resources.
    pub async fn delete_room(
        &self,
        current_user: RequestUser,
        room_id: RoomId,
        force_delete_reference_if_external_services_fail: bool,
    ) -> Result<(), ApiError> {
        self.backend
            .read()
            .await
            .delete_room(
                current_user,
                room_id,
                force_delete_reference_if_external_services_fail,
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

    /// Get the assets associated with a room
    pub async fn get_room_assets(
        &self,
        room_id: RoomId,
        pagination: &PagePaginationQuery,
    ) -> Result<(RoomsByRoomIdAssetsGetResponseBody, i64), ApiError> {
        self.backend
            .read()
            .await
            .get_room_assets(room_id, pagination)
            .await
    }

    /// Get a specific asset inside a room.
    pub async fn get_room_asset(
        &self,
        room_id: RoomId,
        asset_id: AssetId,
    ) -> Result<ByStreamExt, ApiError> {
        self.backend
            .read()
            .await
            .get_room_asset(room_id, asset_id)
            .await
    }

    /// Create an asset for a room from an uploaded file
    pub async fn create_room_asset(
        &self,
        room_id: RoomId,
        filename: NewAssetFileName,
        namespace: Option<ModuleId>,
        data: Box<dyn Stream<Item = Result<Bytes, ObjectStorageError>> + Unpin>,
    ) -> Result<AssetResource, ApiError> {
        self.backend
            .read()
            .await
            .create_room_asset(room_id, filename, namespace, data)
            .await
    }

    /// Delete an asset from a room
    pub async fn delete_room_asset(
        &self,
        room_id: RoomId,
        asset_id: AssetId,
    ) -> Result<(), ApiError> {
        self.backend
            .read()
            .await
            .delete_room_asset(room_id, asset_id)
            .await
    }

    /// Create a new invite
    pub async fn create_invite(
        &self,
        current_user: RequestUser,
        room_id: RoomId,
        new_invite: PostInviteRequestBody,
    ) -> Result<InviteResource, ApiError> {
        self.backend
            .read()
            .await
            .create_invite(current_user, room_id, new_invite)
            .await
    }
    /// Get all invites for a room
    pub async fn get_invites(
        &self,
        room_id: RoomId,
        pagination: &PagePaginationQuery,
    ) -> Result<(GetRoomsInvitesResponseBody, i64), ApiError> {
        self.backend
            .read()
            .await
            .get_invites(room_id, pagination)
            .await
    }

    /// Get a room invite
    pub async fn get_invite(
        &self,
        room_id: RoomId,
        invite_code: InviteCode,
    ) -> Result<InviteResource, ApiError> {
        self.backend
            .read()
            .await
            .get_invite(room_id, invite_code)
            .await
    }

    /// Update an invite code
    pub async fn update_invite(
        &self,
        current_user: RequestUser,
        room_id: RoomId,
        invite_code: InviteCode,
        body: PutInviteRequestBody,
    ) -> Result<InviteResource, ApiError> {
        self.backend
            .read()
            .await
            .update_invite(current_user, room_id, invite_code, body)
            .await
    }

    /// Delete an invite code
    pub async fn delete_invite(
        &self,
        current_user: RequestUser,
        room_id: RoomId,
        invite_code: InviteCode,
    ) -> Result<(), ApiError> {
        self.backend
            .read()
            .await
            .delete_invite(current_user, room_id, invite_code)
            .await
    }

    /// Verify an invite code
    pub async fn verify_invite_code(
        &self,
        data: PostInviteVerifyRequestBody,
    ) -> Result<PostInviteVerifyResponseBody, ApiError> {
        self.backend.read().await.verify_invite_code(data).await
    }

    /// Get the sip config for the specified room.
    pub async fn get_sip_config(&self, room_id: RoomId) -> Result<SipConfigResource, ApiError> {
        self.backend.read().await.get_sip_config(room_id).await
    }

    /// Modify the sip configuration of a room. A new sip configuration is created
    /// if none was set before.
    pub async fn set_sip_config(
        &self,
        room_id: RoomId,
        modify_sip_config: PutSipConfigRequestBody,
    ) -> Result<(SipConfigResource, bool), ApiError> {
        self.backend
            .read()
            .await
            .set_sip_config(room_id, modify_sip_config)
            .await
    }

    /// Delete the SIP configuration of a room.
    pub async fn delete_sip_config(&self, room_id: RoomId) -> Result<(), ApiError> {
        self.backend.read().await.delete_sip_config(room_id).await
    }

    /// Add an event to the current user's favorites
    pub async fn add_event_to_favorites(
        &self,
        current_user: RequestUser,
        event_id: EventId,
    ) -> Result<bool, ApiError> {
        self.backend
            .read()
            .await
            .add_event_to_favorites(current_user, event_id)
            .await
    }

    /// Remove an event from the current user's favorites
    pub async fn remove_event_from_favorites(
        &self,
        current_user: RequestUser,
        event_id: EventId,
    ) -> Result<(), ApiError> {
        self.backend
            .read()
            .await
            .remove_event_from_favorites(current_user, event_id)
            .await
    }
}
