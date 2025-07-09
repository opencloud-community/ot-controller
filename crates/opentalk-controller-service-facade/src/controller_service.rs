// SPDX-FileCopyrightText: OpenTalk GmbH <mail@opentalk.eu>
//
// SPDX-License-Identifier: EUPL-1.2

use std::sync::Arc;

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

    /// Start a signaling session as a registered user
    pub async fn start_room_session(
        &self,
        current_user: RequestUser,
        room_id: RoomId,
        request: PostRoomsStartRequestBody,
    ) -> Result<RoomsStartResponseBody, ApiError> {
        self.backend
            .read()
            .await
            .start_room_session(current_user, room_id, request)
            .await
    }

    /// Start a signaling session for an invitation code
    pub async fn start_invited_room_session(
        &self,
        room_id: RoomId,
        request: PostRoomsStartInvitedRequestBody,
    ) -> Result<RoomsStartResponseBody, ApiError> {
        self.backend
            .read()
            .await
            .start_invited_room_session(room_id, request)
            .await
    }

    /// Start a roomserver signaling session as a registered user
    pub async fn start_roomserver_room_session(
        &self,
        current_user: RequestUser,
        room_id: RoomId,
        request: PostRoomsRoomserverStartRequestBody,
    ) -> Result<RoomserverStartResponseBody, ApiError> {
        self.backend
            .read()
            .await
            .start_roomserver_room_session(current_user, room_id, request)
            .await
    }

    /// Start a roomserver signaling session for an invitation code
    pub async fn start_invited_roomserver_room_session(
        &self,
        room_id: RoomId,
        request: PostRoomsRoomserverStartInvitedRequestBody,
    ) -> Result<RoomserverStartResponseBody, ApiError> {
        self.backend
            .read()
            .await
            .start_invited_roomserver_room_session(room_id, request)
            .await
    }

    /// Starts a signaling session for recording
    pub async fn start_recording(
        &self,
        body: PostRecordingStartRequestBody,
    ) -> Result<PostServiceStartResponseBody, ApiError> {
        self.backend.read().await.start_recording(body).await
    }

    /// Starts a signaling session for call-in
    pub async fn start_call_in(
        &self,
        request: PostCallInStartRequestBody,
    ) -> Result<PostServiceStartResponseBody, ApiError> {
        self.backend.read().await.start_call_in(request).await
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

    /// Create a new event
    pub async fn new_event(
        &self,
        current_user: RequestUser,
        event: PostEventsBody,
        query: EventOptionsQuery,
    ) -> Result<EventResource, ApiError> {
        self.backend
            .read()
            .await
            .new_event(current_user, event, query)
            .await
    }

    /// Get a list of events accessible by the requesting user
    pub async fn get_events(
        &self,
        current_user: RequestUser,
        query: GetEventsQuery,
    ) -> Result<(Vec<EventOrException>, Option<String>, Option<String>), ApiError> {
        self.backend
            .read()
            .await
            .get_events(current_user, query)
            .await
    }

    /// Get an event
    pub async fn get_event(
        &self,
        current_user: RequestUser,
        event_id: EventId,
        query: GetEventQuery,
    ) -> Result<EventResource, ApiError> {
        self.backend
            .read()
            .await
            .get_event(current_user, event_id, query)
            .await
    }

    /// Patch an event
    pub async fn patch_event(
        &self,
        current_user: RequestUser,
        event_id: EventId,
        query: PatchEventQuery,
        patch: PatchEventBody,
    ) -> Result<Option<EventResource>, ApiError> {
        self.backend
            .read()
            .await
            .patch_event(current_user, event_id, query, patch)
            .await
    }

    /// Delete an event and its owned resources, including the associated room.
    pub async fn delete_event(
        &self,
        current_user: RequestUser,
        event_id: EventId,
        query: DeleteEventsQuery,
    ) -> Result<(), ApiError> {
        self.backend
            .read()
            .await
            .delete_event(current_user, event_id, query)
            .await
    }

    /// Get a list of the instances of an event
    pub async fn get_event_instances(
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
    > {
        self.backend
            .read()
            .await
            .get_event_instances(current_user, event_id, query)
            .await
    }

    /// Get an event instance
    pub async fn get_event_instance(
        &self,
        current_user: &RequestUser,
        path: EventInstancePath,
        query: EventInstanceQuery,
    ) -> Result<GetEventInstanceResponseBody, ApiError> {
        self.backend
            .read()
            .await
            .get_event_instance(current_user, path, query)
            .await
    }

    /// Modifies an event instance
    pub async fn patch_event_instance(
        &self,
        current_user: RequestUser,
        path: EventInstancePath,
        query: EventInstanceQuery,
        patch: PatchEventInstanceBody,
    ) -> Result<Option<EventInstance>, ApiError> {
        self.backend
            .read()
            .await
            .patch_event_instance(current_user, path, query, patch)
            .await
    }

    /// Get the invites for an event
    pub async fn get_invites_for_event(
        &self,
        current_user: RequestUser,
        event_id: EventId,
        query: GetEventsInvitesQuery,
    ) -> Result<(Vec<EventInvitee>, i64, i64, i64), ApiError> {
        self.backend
            .read()
            .await
            .get_invites_for_event(current_user, event_id, query)
            .await
    }

    /// Create a new invite to an event
    pub async fn create_invite_to_event(
        &self,
        current_user: RequestUser,
        event_id: EventId,
        query: PostEventInviteQuery,
        create_invite: PostEventInviteBody,
    ) -> Result<bool, ApiError> {
        self.backend
            .read()
            .await
            .create_invite_to_event(current_user, event_id, query, create_invite)
            .await
    }

    /// Patch an event invite with the provided fields
    pub async fn update_invite_to_event(
        &self,
        current_user: &RequestUser,
        event_id: EventId,
        user_id: UserId,
        update_invite: &PatchInviteBody,
    ) -> Result<(), ApiError> {
        self.backend
            .read()
            .await
            .update_invite_to_event(current_user, event_id, user_id, update_invite)
            .await
    }

    /// Patch an event email invite with the provided fields
    pub async fn update_email_invite_to_event(
        &self,
        current_user: &RequestUser,
        event_id: EventId,
        update_invite: &PatchEmailInviteBody,
    ) -> Result<(), ApiError> {
        self.backend
            .read()
            .await
            .update_email_invite_to_event(current_user, event_id, update_invite)
            .await
    }

    /// Delete an invite from an event
    pub async fn delete_invite_to_event(
        &self,
        current_user: RequestUser,
        path: DeleteEventInvitePath,
        query: EventOptionsQuery,
    ) -> Result<(), ApiError> {
        self.backend
            .read()
            .await
            .delete_invite_to_event(current_user, path, query)
            .await
    }

    /// Delete an invite from an event
    pub async fn delete_email_invite_to_event(
        &self,
        current_user: RequestUser,
        event_id: EventId,
        email: EmailAddress,
        query: EventOptionsQuery,
    ) -> Result<(), ApiError> {
        self.backend
            .read()
            .await
            .delete_email_invite_to_event(current_user, event_id, email, query)
            .await
    }

    /// Get information about pending invites
    pub async fn get_event_invites_pending(
        &self,
        user_id: UserId,
    ) -> Result<GetEventInvitesPendingResponseBody, ApiError> {
        self.backend
            .read()
            .await
            .get_event_invites_pending(user_id)
            .await
    }

    /// Accept an invite to an event
    pub async fn accept_event_invite(
        &self,
        user_id: UserId,
        event_id: EventId,
    ) -> Result<(), ApiError> {
        self.backend
            .read()
            .await
            .accept_event_invite(user_id, event_id)
            .await
    }

    /// Decline an invite to an event
    pub async fn decline_event_invite(
        &self,
        user_id: UserId,
        event_id: EventId,
    ) -> Result<(), ApiError> {
        self.backend
            .read()
            .await
            .decline_event_invite(user_id, event_id)
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

    /// Lists the streaming targets of a room
    pub async fn get_streaming_targets(
        &self,
        user_id: UserId,
        room_id: RoomId,
        pagination: &PagePaginationQuery,
    ) -> Result<GetRoomStreamingTargetsResponseBody, ApiError> {
        self.backend
            .read()
            .await
            .get_streaming_targets(user_id, room_id, pagination)
            .await
    }

    /// Creates a new streaming target
    pub async fn post_streaming_target(
        &self,
        current_user: RequestUser,
        room_id: RoomId,
        query: StreamingTargetOptionsQuery,
        streaming_target: StreamingTarget,
    ) -> Result<PostRoomStreamingTargetResponseBody, ApiError> {
        self.backend
            .read()
            .await
            .post_streaming_target(current_user, room_id, query, streaming_target)
            .await
    }

    /// Gets a streaming target
    pub async fn get_streaming_target(
        &self,
        user_id: UserId,
        path_params: RoomAndStreamingTargetId,
    ) -> Result<GetRoomStreamingTargetResponseBody, ApiError> {
        self.backend
            .read()
            .await
            .get_streaming_target(user_id, path_params)
            .await
    }

    /// Updates a streaming target
    pub async fn patch_streaming_target(
        &self,
        current_user: RequestUser,
        path_params: RoomAndStreamingTargetId,
        query: StreamingTargetOptionsQuery,
        streaming_target: PatchRoomStreamingTargetRequestBody,
    ) -> Result<PatchRoomStreamingTargetResponseBody, ApiError> {
        self.backend
            .read()
            .await
            .patch_streaming_target(current_user, path_params, query, streaming_target)
            .await
    }

    /// Deletes a streaming target
    pub async fn delete_streaming_target(
        &self,
        current_user: RequestUser,
        path_params: RoomAndStreamingTargetId,
        query: StreamingTargetOptionsQuery,
    ) -> Result<(), ApiError> {
        self.backend
            .read()
            .await
            .delete_streaming_target(current_user, path_params, query)
            .await
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

    /// Get the shared folder for an event
    pub async fn get_shared_folder_for_event(
        &self,
        current_user: RequestUser,
        event_id: EventId,
    ) -> Result<SharedFolder, ApiError> {
        self.backend
            .read()
            .await
            .get_shared_folder_for_event(current_user, event_id)
            .await
    }

    /// Create a shared folder for an event
    pub async fn put_shared_folder_for_event(
        &self,
        current_user: RequestUser,
        event_id: EventId,
        query: PutSharedFolderQuery,
    ) -> Result<(SharedFolder, bool), ApiError> {
        self.backend
            .read()
            .await
            .put_shared_folder_for_event(current_user, event_id, query)
            .await
    }

    /// Delete the shared folder of an event
    pub async fn delete_shared_folder_for_event(
        &self,
        current_user: RequestUser,
        event_id: EventId,
        query: DeleteSharedFolderQuery,
    ) -> Result<(), ApiError> {
        self.backend
            .read()
            .await
            .delete_shared_folder_for_event(current_user, event_id, query)
            .await
    }

    /// Patch the current user's profile.
    pub async fn patch_me(
        &self,
        current_user: RequestUser,
        patch: PatchMeRequestBody,
    ) -> Result<Option<PrivateUserProfile>, ApiError> {
        self.backend
            .read()
            .await
            .patch_me(current_user, patch)
            .await
    }

    /// Get the current user's profile.
    pub async fn get_me(&self, current_user: RequestUser) -> Result<PrivateUserProfile, ApiError> {
        self.backend.read().await.get_me(current_user).await
    }

    /// Get the current user tariff information.
    pub async fn get_my_tariff(
        &self,
        current_user: RequestUser,
    ) -> Result<TariffResource, ApiError> {
        self.backend.read().await.get_my_tariff(current_user).await
    }

    /// Get the assets associated with the user.
    pub async fn get_my_assets(
        &self,
        current_user: RequestUser,
        sorting: AssetSortingQuery,
        pagination: &PagePaginationQuery,
    ) -> Result<(GetUserAssetsResponseBody, i64), ApiError> {
        self.backend
            .read()
            .await
            .get_my_assets(current_user, sorting, pagination)
            .await
    }

    /// Get a user's public profile.
    pub async fn get_user(
        &self,
        current_user: RequestUser,
        user_id: UserId,
    ) -> Result<PublicUserProfile, ApiError> {
        self.backend
            .read()
            .await
            .get_user(current_user, user_id)
            .await
    }

    /// Find users.
    pub async fn find_users(
        &self,
        current_user: RequestUser,
        query: GetFindQuery,
    ) -> Result<GetFindResponseBody, ApiError> {
        self.backend
            .read()
            .await
            .find_users(current_user, query)
            .await
    }
}
