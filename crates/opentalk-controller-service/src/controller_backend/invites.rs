// SPDX-FileCopyrightText: OpenTalk GmbH <mail@opentalk.eu>
//
// SPDX-License-Identifier: EUPL-1.2

use chrono::Utc;
use kustos::policies_builder::PoliciesBuilder;
use opentalk_controller_service_facade::RequestUser;
use opentalk_controller_utils::{
    deletion::room::associated_resource_ids_for_invite, CaptureApiError,
};
use opentalk_db_storage::{
    invites::{Invite, NewInvite, UpdateInvite},
    rooms::Room,
    users::User,
};
use opentalk_types_api_v1::{
    error::ApiError,
    pagination::PagePaginationQuery,
    rooms::by_room_id::invites::{
        GetRoomsInvitesResponseBody, InviteResource, PostInviteRequestBody,
        PostInviteVerifyRequestBody, PostInviteVerifyResponseBody, PutInviteRequestBody,
    },
    users::PublicUserProfile,
};
use opentalk_types_common::rooms::{invite_codes::InviteCode, RoomId};

use crate::{controller_backend::RoomsPoliciesBuilderExt, ControllerBackend, ToUserProfile};

impl ControllerBackend {
    pub(crate) async fn create_invite(
        &self,
        current_user: RequestUser,
        room_id: RoomId,
        new_invite: PostInviteRequestBody,
    ) -> Result<InviteResource, CaptureApiError> {
        let settings = self.settings_provider.get_raw();
        let mut conn = self.db.get_conn().await?;

        let invite = NewInvite {
            active: true,
            created_by: current_user.id,
            updated_by: current_user.id,
            room: room_id,
            expiration: new_invite.expiration,
        }
        .insert(&mut conn)
        .await?;

        let policies = PoliciesBuilder::new()
            // Grant invitee access
            .grant_invite_access(invite.id)
            .room_guest_read_access(room_id)
            .finish();

        self.authz.add_policies(policies).await?;

        let created_by = current_user.to_public_user_profile(&settings);
        let updated_by = current_user.to_public_user_profile(&settings);

        let invite = Invite::into_invite_resource(invite, created_by, updated_by);

        Ok(invite)
    }

    pub(crate) async fn get_invites(
        &self,
        room_id: RoomId,
        pagination: &PagePaginationQuery,
    ) -> Result<(GetRoomsInvitesResponseBody, i64), CaptureApiError> {
        let settings = self.settings_provider.get_raw();
        let mut conn = self.db.get_conn().await?;

        let room = Room::get(&mut conn, room_id).await?;

        let (invites_with_users, total_invites) = Invite::get_all_for_room_with_users_paginated(
            &mut conn,
            room.id,
            pagination.per_page,
            pagination.page,
        )
        .await?;

        let invites = invites_with_users
            .into_iter()
            .map(|(db_invite, created_by, updated_by)| {
                let created_by = created_by.to_public_user_profile(&settings);
                let updated_by = updated_by.to_public_user_profile(&settings);

                Invite::into_invite_resource(db_invite, created_by, updated_by)
            })
            .collect::<Vec<InviteResource>>();

        Ok((GetRoomsInvitesResponseBody(invites), total_invites))
    }

    pub(crate) async fn get_invite(
        &self,
        room_id: RoomId,
        invite_code: InviteCode,
    ) -> Result<InviteResource, CaptureApiError> {
        let settings = self.settings_provider.get_raw();
        let mut conn = self.db.get_conn().await?;

        let (invite, created_by, updated_by) =
            Invite::get_with_users(&mut conn, invite_code).await?;

        if invite.room != room_id {
            return Err(ApiError::not_found().into());
        }

        let created_by = created_by.to_public_user_profile(&settings);
        let updated_by = updated_by.to_public_user_profile(&settings);

        Ok(Invite::into_invite_resource(invite, created_by, updated_by))
    }

    pub(crate) async fn update_invite(
        &self,
        current_user: RequestUser,
        room_id: RoomId,
        invite_code: InviteCode,
        body: PutInviteRequestBody,
    ) -> Result<InviteResource, CaptureApiError> {
        let settings = self.settings_provider.get_raw();
        let mut conn = self.db.get_conn().await?;

        let invite = Invite::get(&mut conn, invite_code).await?;

        if invite.room != room_id {
            return Err(ApiError::not_found().into());
        }

        let created_by = User::get(&mut conn, invite.created_by).await?;

        let now = Utc::now();
        let changeset = UpdateInvite {
            updated_by: Some(current_user.id),
            updated_at: Some(now),
            expiration: Some(body.expiration),
            active: None,
            room: None,
        };

        let invite = changeset.apply(&mut conn, room_id, invite_code).await?;

        let created_by = created_by.to_public_user_profile(&settings);
        let updated_by = current_user.to_public_user_profile(&settings);

        Ok(Invite::into_invite_resource(invite, created_by, updated_by))
    }

    pub(crate) async fn delete_invite(
        &self,
        current_user: RequestUser,
        room_id: RoomId,
        invite_code: InviteCode,
    ) -> Result<(), CaptureApiError> {
        let mut conn = self.db.get_conn().await?;

        let changeset = UpdateInvite {
            updated_by: Some(current_user.id),
            updated_at: Some(Utc::now()),
            expiration: None,
            active: Some(false),
            room: None,
        };

        _ = changeset.apply(&mut conn, room_id, invite_code).await?;

        let associated_resources = Vec::from_iter(associated_resource_ids_for_invite(room_id));
        let _ = self
            .authz
            .remove_all_invite_permission_for_resources(invite_code, associated_resources)
            .await?;

        Ok(())
    }

    pub(crate) async fn verify_invite_code(
        &self,
        data: PostInviteVerifyRequestBody,
    ) -> Result<PostInviteVerifyResponseBody, CaptureApiError> {
        let mut conn = self.db.get_conn().await?;

        let invite = Invite::get(&mut conn, data.invite_code).await?;
        let room = Room::get(&mut conn, invite.room).await?;

        if invite.active {
            if let Some(expiration) = invite.expiration {
                if expiration <= Utc::now() {
                    // Do not leak the existence of the invite when it is expired
                    return Err(ApiError::not_found().into());
                }
            }
            Ok(PostInviteVerifyResponseBody {
                room_id: invite.room,
                password_required: room.password.is_some(),
            })
        } else {
            // TODO(r.floren) Do we want to return something else here?
            Err(ApiError::not_found().into())
        }
    }
}

trait IntoInviteResource {
    fn into_invite_resource(
        invite: Invite,
        created_by: PublicUserProfile,
        updated_by: PublicUserProfile,
    ) -> InviteResource;
}

impl IntoInviteResource for Invite {
    fn into_invite_resource(
        invite: Invite,
        created_by: PublicUserProfile,
        updated_by: PublicUserProfile,
    ) -> InviteResource {
        InviteResource {
            invite_code: invite.id,
            created: invite.created_at,
            created_by,
            updated: invite.updated_at,
            updated_by,
            room_id: invite.room,
            active: invite.active,
            expiration: invite.expiration,
        }
    }
}
