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
    assets::{AssetResource, AssetSortingQuery},
    auth::GetLoginResponseBody,
    error::ApiError,
    events::{DeleteSharedFolderQuery, PutSharedFolderQuery, StreamingTargetOptionsQuery},
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
    modules::ModuleId,
    rooms::{invite_codes::InviteCode, RoomId, RoomPassword},
    shared_folders::SharedFolder,
    streaming::StreamingTarget,
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

    /// Lists the streaming targets of a room
    async fn get_streaming_targets(
        &self,
        user_id: UserId,
        room_id: RoomId,
        pagination: &PagePaginationQuery,
    ) -> Result<GetRoomStreamingTargetsResponseBody, ApiError>;

    /// Creates a new streaming target
    async fn post_streaming_target(
        &self,
        current_user: RequestUser,
        room_id: RoomId,
        query: StreamingTargetOptionsQuery,
        streaming_target: StreamingTarget,
    ) -> Result<PostRoomStreamingTargetResponseBody, ApiError>;

    /// Gets a streaming target
    async fn get_streaming_target(
        &self,
        user_id: UserId,
        path_params: RoomAndStreamingTargetId,
    ) -> Result<GetRoomStreamingTargetResponseBody, ApiError>;

    /// Updates a streaming target
    async fn patch_streaming_target(
        &self,
        current_user: RequestUser,
        path_params: RoomAndStreamingTargetId,
        query: StreamingTargetOptionsQuery,
        update_streaming_target: PatchRoomStreamingTargetRequestBody,
    ) -> Result<PatchRoomStreamingTargetResponseBody, ApiError>;

    /// Deletes a streaming target
    async fn delete_streaming_target(
        &self,
        current_user: RequestUser,
        path_params: RoomAndStreamingTargetId,
        query: StreamingTargetOptionsQuery,
    ) -> Result<(), ApiError>;

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

    /// Get the shared folder for an event
    async fn get_shared_folder_for_event(
        &self,
        current_user: RequestUser,
        event_id: EventId,
    ) -> Result<SharedFolder, ApiError>;

    /// Create a shared folder for an event
    async fn put_shared_folder_for_event(
        &self,
        current_user: RequestUser,
        event_id: EventId,
        query: PutSharedFolderQuery,
    ) -> Result<(SharedFolder, bool), ApiError>;

    /// Delete the shared folder of an event
    async fn delete_shared_folder_for_event(
        &self,
        current_user: RequestUser,
        event_id: EventId,
        query: DeleteSharedFolderQuery,
    ) -> Result<(), ApiError>;

    /// Patch the current user's profile.
    async fn patch_me(
        &self,
        current_user: RequestUser,
        patch: PatchMeRequestBody,
    ) -> Result<Option<PrivateUserProfile>, ApiError>;

    /// Get the current user's profile.
    async fn get_me(&self, current_user: RequestUser) -> Result<PrivateUserProfile, ApiError>;

    /// Get the current user tariff information.
    async fn get_my_tariff(&self, current_user: RequestUser) -> Result<TariffResource, ApiError>;

    /// Get the assets associated with the user.
    async fn get_my_assets(
        &self,
        current_user: RequestUser,
        sorting: AssetSortingQuery,
        pagination: &PagePaginationQuery,
    ) -> Result<(GetUserAssetsResponseBody, i64), ApiError>;

    /// Get a user's public profile.
    async fn get_user(
        &self,
        current_user: RequestUser,
        user_id: UserId,
    ) -> Result<PublicUserProfile, ApiError>;

    /// Find users.
    async fn find_users(
        &self,
        current_user: RequestUser,
        query: GetFindQuery,
    ) -> Result<GetFindResponseBody, ApiError>;
}
