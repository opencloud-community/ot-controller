// SPDX-FileCopyrightText: OpenTalk GmbH <mail@opentalk.eu>
//
// SPDX-License-Identifier: EUPL-1.2

use async_trait::async_trait;
use bytes::Bytes;
use futures_core::Stream;
use opentalk_signaling_core::{
    ObjectStorageError,
    assets::{ByStreamExt, NewAssetFileName},
};
use opentalk_types_api_v1::{
    assets::{AssetResource, AssetSortingQuery},
    auth::GetLoginResponseBody,
    error::ApiError,
    events::{
        DeleteEventInvitePath, DeleteEventsQuery, DeleteSharedFolderQuery, EventInstance,
        EventInstancePath, EventInstanceQuery, EventInvitee, EventOptionsQuery, EventOrException,
        EventResource, GetEventInstanceResponseBody, GetEventInstancesQuery,
        GetEventInstancesResponseBody, GetEventQuery, GetEventsQuery, PatchEmailInviteBody,
        PatchEventBody, PatchEventInstanceBody, PatchEventQuery, PatchInviteBody,
        PostEventInviteBody, PostEventInviteQuery, PostEventsBody, PutSharedFolderQuery,
        StreamingTargetOptionsQuery, by_event_id::invites::GetEventsInvitesQuery,
    },
    pagination::PagePaginationQuery,
    rooms::{
        GetRoomsResponseBody, RoomResource,
        by_room_id::{
            GetRoomEventResponseBody, PostRoomsRoomserverStartInvitedRequestBody,
            PostRoomsRoomserverStartRequestBody, PostRoomsStartInvitedRequestBody,
            PostRoomsStartRequestBody, RoomsStartResponseBody, RoomserverStartResponseBody,
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
        },
    },
    services::{
        PostServiceStartResponseBody, call_in::PostCallInStartRequestBody,
        recording::PostRecordingStartRequestBody,
    },
    users::{
        GetEventInvitesPendingResponseBody, GetFindQuery, GetFindResponseBody,
        GetUserAssetsResponseBody, PrivateUserProfile, PublicUserProfile, me::PatchMeRequestBody,
    },
};
use opentalk_types_common::{
    assets::AssetId,
    email::EmailAddress,
    events::EventId,
    modules::ModuleId,
    rooms::{RoomId, RoomPassword, invite_codes::InviteCode},
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

    /// Start a signaling session as a registered user
    async fn start_room_session(
        &self,
        current_user: RequestUser,
        room_id: RoomId,
        request: PostRoomsStartRequestBody,
    ) -> Result<RoomsStartResponseBody, ApiError>;

    /// Start a signaling session for an invitation code
    async fn start_invited_room_session(
        &self,
        room_id: RoomId,
        request: PostRoomsStartInvitedRequestBody,
    ) -> Result<RoomsStartResponseBody, ApiError>;

    /// Start a roomserver signaling session as a registered user
    async fn start_roomserver_room_session(
        &self,
        current_user: RequestUser,
        room_id: RoomId,
        request: PostRoomsRoomserverStartRequestBody,
    ) -> Result<RoomserverStartResponseBody, ApiError>;

    /// Start a roomserver signaling session for an invitation code
    async fn start_invited_roomserver_room_session(
        &self,
        room_id: RoomId,
        request: PostRoomsRoomserverStartInvitedRequestBody,
    ) -> Result<RoomserverStartResponseBody, ApiError>;

    /// Starts a signaling session for recording
    async fn start_recording(
        &self,
        body: PostRecordingStartRequestBody,
    ) -> Result<PostServiceStartResponseBody, ApiError>;

    /// Starts a signaling session for call-in
    async fn start_call_in(
        &self,
        request: PostCallInStartRequestBody,
    ) -> Result<PostServiceStartResponseBody, ApiError>;

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

    /// Create a new event
    async fn new_event(
        &self,
        current_user: RequestUser,
        event: PostEventsBody,
        query: EventOptionsQuery,
    ) -> Result<EventResource, ApiError>;

    /// Get a list of events accessible by the requesting user
    async fn get_events(
        &self,
        current_user: RequestUser,
        query: GetEventsQuery,
    ) -> Result<(Vec<EventOrException>, Option<String>, Option<String>), ApiError>;

    /// Get an event
    async fn get_event(
        &self,
        current_user: RequestUser,
        event_id: EventId,
        query: GetEventQuery,
    ) -> Result<EventResource, ApiError>;

    /// Patch an event
    async fn patch_event(
        &self,
        current_user: RequestUser,
        event_id: EventId,
        query: PatchEventQuery,
        patch: PatchEventBody,
    ) -> Result<Option<EventResource>, ApiError>;

    /// Delete an event and its owned resources, including the associated room.
    async fn delete_event(
        &self,
        current_user: RequestUser,
        event_id: EventId,
        query: DeleteEventsQuery,
    ) -> Result<(), ApiError>;

    /// Get a list of the instances of an event
    async fn get_event_instances(
        &self,
        current_user: &RequestUser,
        event_id: EventId,
        query: GetEventInstancesQuery,
    ) -> Result<
        (
            GetEventInstancesResponseBody,
            Option<String>,
            Option<String>,
        ),
        ApiError,
    >;

    /// Get an event instance
    async fn get_event_instance(
        &self,
        current_user: &RequestUser,
        path: EventInstancePath,
        query: EventInstanceQuery,
    ) -> Result<GetEventInstanceResponseBody, ApiError>;

    /// Modifies an event instance
    async fn patch_event_instance(
        &self,
        current_user: RequestUser,
        path: EventInstancePath,
        query: EventInstanceQuery,
        patch: PatchEventInstanceBody,
    ) -> Result<Option<EventInstance>, ApiError>;

    /// Get the invites for an event
    async fn get_invites_for_event(
        &self,
        current_user: RequestUser,
        event_id: EventId,
        query: GetEventsInvitesQuery,
    ) -> Result<(Vec<EventInvitee>, i64, i64, i64), ApiError>;

    /// Create a new invite to an event
    async fn create_invite_to_event(
        &self,
        current_user: RequestUser,
        event_id: EventId,
        query: PostEventInviteQuery,
        create_invite: PostEventInviteBody,
    ) -> Result<bool, ApiError>;

    /// Patch an event invite with the provided fields
    async fn update_invite_to_event(
        &self,
        current_user: &RequestUser,
        event_id: EventId,
        user_id: UserId,
        update_invite: &PatchInviteBody,
    ) -> Result<(), ApiError>;

    /// Patch an event email invite with the provided fields
    async fn update_email_invite_to_event(
        &self,
        current_user: &RequestUser,
        event_id: EventId,
        update_invite: &PatchEmailInviteBody,
    ) -> Result<(), ApiError>;

    /// Delete an invite from an event
    async fn delete_invite_to_event(
        &self,
        current_user: RequestUser,
        path: DeleteEventInvitePath,
        query: EventOptionsQuery,
    ) -> Result<(), ApiError>;

    /// Delete an invite from an event
    async fn delete_email_invite_to_event(
        &self,
        current_user: RequestUser,
        event_id: EventId,
        email: EmailAddress,
        query: EventOptionsQuery,
    ) -> Result<(), ApiError>;

    /// Get information about pending invites
    async fn get_event_invites_pending(
        &self,
        user_id: UserId,
    ) -> Result<GetEventInvitesPendingResponseBody, ApiError>;

    /// Accept an invite to an event
    async fn accept_event_invite(&self, user_id: UserId, event_id: EventId)
    -> Result<(), ApiError>;

    /// Decline an invite to an event
    async fn decline_event_invite(
        &self,
        user_id: UserId,
        event_id: EventId,
    ) -> Result<(), ApiError>;

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
