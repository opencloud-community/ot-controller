// SPDX-FileCopyrightText: OpenTalk GmbH <mail@opentalk.eu>
//
// SPDX-License-Identifier: EUPL-1.2

use async_trait::async_trait;
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

use crate::RequestUser;

/// Trait implemented by OpenTalk controller service backends
#[async_trait(?Send)]
pub trait OpenTalkControllerServiceBackend: Send + Sync {
    /// Get the configured OIDC provider
    async fn get_login(&self) -> GetLoginResponseBody;

    /// Get all accessible rooms
    async fn get_rooms(
        &self,
        current_user_id: UserId,
        pagination: &PagePaginationQuery,
    ) -> Result<(GetRoomsResponseBody, i64), ApiError>;

    /// Create a new room
    async fn create_room(
        &self,
        current_user: RequestUser,
        password: Option<RoomPassword>,
        enable_sip: bool,
        waiting_room: bool,
        e2e_encryption: bool,
    ) -> Result<RoomResource, ApiError>;

    /// Patch a room with the provided fields
    async fn patch_room(
        &self,
        current_user: RequestUser,
        room_id: RoomId,
        password: Option<Option<RoomPassword>>,
        waiting_room: Option<bool>,
        e2e_encryption: Option<bool>,
    ) -> Result<RoomResource, ApiError>;

    /// Delete a room and its owned resources.
    async fn delete_room(
        &self,
        current_user: RequestUser,
        room_id: RoomId,
        force_delete_reference_if_external_services_fail: bool,
    ) -> Result<(), ApiError>;

    /// Get a room
    async fn get_room(&self, room_id: &RoomId) -> Result<RoomResource, ApiError>;

    /// Get a room's tariff
    async fn get_room_tariff(&self, room_id: &RoomId) -> Result<TariffResource, ApiError>;

    /// Get a room's event
    async fn get_room_event(&self, room_id: &RoomId) -> Result<GetRoomEventResponseBody, ApiError>;

    /// Get the assets associated with a room.
    async fn get_room_assets(
        &self,
        room_id: RoomId,
        pagination: &PagePaginationQuery,
    ) -> Result<(RoomsByRoomIdAssetsGetResponseBody, i64), ApiError>;

    /// Get a specific asset inside a room.
    async fn get_room_asset(
        &self,
        room_id: RoomId,
        asset_id: AssetId,
    ) -> Result<ByStreamExt, ApiError>;

    /// Create an asset for a room from an uploaded file.
    async fn create_room_asset(
        &self,
        room_id: RoomId,
        filename: NewAssetFileName,
        namespace: Option<ModuleId>,
        data: Box<dyn Stream<Item = Result<Bytes, ObjectStorageError>> + Unpin>,
    ) -> Result<AssetResource, ApiError>;

    /// Delete an asset from a room.
    async fn delete_room_asset(&self, room_id: RoomId, asset_id: AssetId) -> Result<(), ApiError>;

    /// Create a new invite
    async fn create_invite(
        &self,
        current_user: RequestUser,
        room_id: RoomId,
        new_invite: PostInviteRequestBody,
    ) -> Result<InviteResource, ApiError>;

    /// Get all invites for a room
    async fn get_invites(
        &self,
        room_id: RoomId,
        pagination: &PagePaginationQuery,
    ) -> Result<(GetRoomsInvitesResponseBody, i64), ApiError>;

    /// Get a room invite
    async fn get_invite(
        &self,
        room_id: RoomId,
        invite_code: InviteCode,
    ) -> Result<InviteResource, ApiError>;

    /// Update an invite code
    async fn update_invite(
        &self,
        current_user: RequestUser,
        room_id: RoomId,
        invite_code: InviteCode,
        body: PutInviteRequestBody,
    ) -> Result<InviteResource, ApiError>;

    /// Delete an invite code
    async fn delete_invite(
        &self,
        current_user: RequestUser,
        room_id: RoomId,
        invite_code: InviteCode,
    ) -> Result<(), ApiError>;

    /// Verify an invite code
    async fn verify_invite_code(
        &self,
        data: PostInviteVerifyRequestBody,
    ) -> Result<PostInviteVerifyResponseBody, ApiError>;

    /// Get the sip config for the specified room.
    async fn get_sip_config(&self, room_id: RoomId) -> Result<SipConfigResource, ApiError>;

    /// Modify the sip configuration of a room. A new sip configuration is created
    /// if none was set before.
    async fn set_sip_config(
        &self,
        room_id: RoomId,
        modify_sip_config: PutSipConfigRequestBody,
    ) -> Result<(SipConfigResource, bool), ApiError>;

    /// Delete the SIP configuration of a room.
    async fn delete_sip_config(&self, room_id: RoomId) -> Result<(), ApiError>;

    /// Add an event to the current user's favorites
    async fn add_event_to_favorites(
        &self,
        current_user: RequestUser,
        event_id: EventId,
    ) -> Result<bool, ApiError>;

    /// Remove an event from the current user's favorites
    async fn remove_event_from_favorites(
        &self,
        current_user: RequestUser,
        event_id: EventId,
    ) -> Result<(), ApiError>;
}
