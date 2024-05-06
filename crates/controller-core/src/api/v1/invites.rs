// SPDX-FileCopyrightText: OpenTalk GmbH <mail@opentalk.eu>
//
// SPDX-License-Identifier: EUPL-1.2

//! Contains invite related REST endpoints.
use actix_web::{
    delete, get, post, put,
    web::{Data, Json, Path, Query, ReqData},
};
use chrono::Utc;
use kustos::{prelude::PoliciesBuilder, Authz};
use opentalk_controller_utils::deletion::room::associated_resource_ids_for_invite;
use opentalk_database::Db;
use opentalk_db_storage::{
    invites::{Invite, NewInvite, UpdateInvite},
    rooms::Room,
    users::User,
};
use opentalk_types::{
    api::{
        error::ApiError,
        v1::{
            invites::{
                CodeVerified, InviteResource, PostInviteRequestBody, PostInviteVerifyRequestBody,
                PutInviteRequestBody, RoomIdAndInviteCode,
            },
            pagination::PagePaginationQuery,
        },
    },
    core::RoomId,
};
use validator::Validate;

use super::{response::NoContent, DefaultApiResult};
use crate::{
    api::{
        responses::{InternalServerError, NotFound, Unauthorized},
        v1::{rooms::RoomsPoliciesBuilderExt, ApiResponse},
    },
    settings::SharedSettingsActix,
};

/// API Endpoint *POST /rooms/{room_id}/invites*
///
/// Uses the provided [`NewInvite`] to create a new invite.
#[post("/rooms/{room_id}/invites")]
pub async fn add_invite(
    settings: SharedSettingsActix,
    db: Data<Db>,
    authz: Data<Authz>,
    current_user: ReqData<User>,
    room_id: Path<RoomId>,
    data: Json<PostInviteRequestBody>,
) -> DefaultApiResult<InviteResource> {
    let settings = settings.load_full();
    let room_id = room_id.into_inner();
    let current_user = current_user.into_inner();
    let new_invite = data.into_inner();

    let mut conn = db.get_conn().await?;

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

    authz.add_policies(policies).await?;

    let created_by = current_user.to_public_user_profile(&settings);
    let updated_by = current_user.to_public_user_profile(&settings);

    let invite = Invite::into_invite_resource(invite, created_by, updated_by);

    Ok(ApiResponse::new(invite))
}

/// API Endpoint *GET /rooms/{room_id}/invites*
///
/// Returns a JSON array of all accessible invites for the given room
#[get("/rooms/{room_id}/invites")]
pub async fn get_invites(
    settings: SharedSettingsActix,
    db: Data<Db>,
    room_id: Path<RoomId>,
    pagination: Query<PagePaginationQuery>,
) -> DefaultApiResult<Vec<InviteResource>> {
    let settings = settings.load_full();
    let room_id = room_id.into_inner();
    let PagePaginationQuery { per_page, page } = pagination.into_inner();

    let mut conn = db.get_conn().await?;

    let room = Room::get(&mut conn, room_id).await?;

    let (invites_with_users, total_invites) =
        Invite::get_all_for_room_with_users_paginated(&mut conn, room.id, per_page, page).await?;

    let invites = invites_with_users
        .into_iter()
        .map(|(db_invite, created_by, updated_by)| {
            let created_by = created_by.to_public_user_profile(&settings);
            let updated_by = updated_by.to_public_user_profile(&settings);

            Invite::into_invite_resource(db_invite, created_by, updated_by)
        })
        .collect::<Vec<InviteResource>>();

    Ok(ApiResponse::new(invites).with_page_pagination(per_page, page, total_invites))
}

/// API Endpoint *GET /rooms/{room_id}/invites/{invite_code}*
///
/// Returns a single invite.
/// Returns 401 Not Found when the user has no access.
#[get("/rooms/{room_id}/invites/{invite_code}")]
pub async fn get_invite(
    settings: SharedSettingsActix,
    db: Data<Db>,
    path_params: Path<RoomIdAndInviteCode>,
) -> DefaultApiResult<InviteResource> {
    let settings = settings.load_full();

    let RoomIdAndInviteCode {
        room_id,
        invite_code,
    } = path_params.into_inner();

    let mut conn = db.get_conn().await?;

    let (invite, created_by, updated_by) = Invite::get_with_users(&mut conn, invite_code).await?;

    if invite.room != room_id {
        return Err(ApiError::not_found());
    }

    let created_by = created_by.to_public_user_profile(&settings);
    let updated_by = updated_by.to_public_user_profile(&settings);

    Ok(ApiResponse::new(Invite::into_invite_resource(
        invite, created_by, updated_by,
    )))
}

/// API Endpoint *PUT /rooms/{room_id}/invites/{invite_code}*
///
/// Uses the provided [`PutInviteRequestBody`] to modify a specified invite.
/// Returns the modified [`InviteResource`]
#[put("/rooms/{room_id}/invites/{invite_code}")]
pub async fn update_invite(
    settings: SharedSettingsActix,
    db: Data<Db>,
    current_user: ReqData<User>,
    path_params: Path<RoomIdAndInviteCode>,
    update_invite: Json<PutInviteRequestBody>,
) -> DefaultApiResult<InviteResource> {
    let settings = settings.load_full();
    let current_user = current_user.into_inner();
    let RoomIdAndInviteCode {
        room_id,
        invite_code,
    } = path_params.into_inner();
    let update_invite = update_invite.into_inner();

    let mut conn = db.get_conn().await?;

    let invite = Invite::get(&mut conn, invite_code).await?;

    if invite.room != room_id {
        return Err(ApiError::not_found());
    }

    let created_by = User::get(&mut conn, invite.created_by).await?;

    let now = chrono::Utc::now();
    let changeset = UpdateInvite {
        updated_by: Some(current_user.id),
        updated_at: Some(now),
        expiration: Some(update_invite.expiration),
        active: None,
        room: None,
    };

    let invite = changeset.apply(&mut conn, room_id, invite_code).await?;

    let created_by = created_by.to_public_user_profile(&settings);
    let updated_by = current_user.to_public_user_profile(&settings);

    Ok(ApiResponse::new(Invite::into_invite_resource(
        invite, created_by, updated_by,
    )))
}

/// API Endpoint *PUT /rooms/{room_id}*
///
/// Deletes the [`Invite`] identified by this resource.
/// Returns 204 No Content
#[delete("/rooms/{room_id}/invites/{invite_code}")]
pub async fn delete_invite(
    db: Data<Db>,
    current_user: ReqData<User>,
    path_params: Path<RoomIdAndInviteCode>,
) -> Result<NoContent, ApiError> {
    let RoomIdAndInviteCode {
        room_id,
        invite_code,
    } = path_params.into_inner();

    let db = db.into_inner();
    let mut conn = db.get_conn().await?;

    let changeset = UpdateInvite {
        updated_by: Some(current_user.id),
        updated_at: Some(Utc::now()),
        expiration: None,
        active: Some(false),
        room: None,
    };

    changeset.apply(&mut conn, room_id, invite_code).await?;

    let authz = Authz::new(db.clone()).await?;

    let associated_resources = Vec::from_iter(associated_resource_ids_for_invite(room_id));
    let _ = authz
        .remove_all_invite_permission_for_resources(invite_code, associated_resources)
        .await?;

    Ok(NoContent)
}

/// Verify an invite code
///
/// Verifies the invite and returns the room url for the invite code
#[utoipa::path(
    request_body = PostInviteVerifyRequestBody,
    responses(
        (
            status = StatusCode::OK,
            description = "Invite is valid, the response body tells the room id",
            body = CodeVerified,
        ),
        (
            status = StatusCode::UNAUTHORIZED,
            response = Unauthorized,
        ),
        (
            status = StatusCode::NOT_FOUND,
            response = NotFound,
        ),
        (
            status = StatusCode::UNPROCESSABLE_ENTITY,
            description = "Invalid body contents received",
        ),
        (
            status = StatusCode::INTERNAL_SERVER_ERROR,
            response = InternalServerError,
        ),
    ),
    security(),
)]
#[post("/invite/verify")]
pub async fn verify_invite_code(
    db: Data<Db>,
    data: Json<PostInviteVerifyRequestBody>,
) -> DefaultApiResult<CodeVerified> {
    let data = data.into_inner();

    data.validate()?;

    let mut conn = db.get_conn().await?;

    let invite = Invite::get(&mut conn, data.invite_code).await?;
    let room = Room::get(&mut conn, invite.room).await?;

    if invite.active {
        if let Some(expiration) = invite.expiration {
            if expiration <= Utc::now() {
                // Do not leak the existence of the invite when it is expired
                return Err(ApiError::not_found());
            }
        }
        Ok(ApiResponse::new(CodeVerified {
            room_id: invite.room,
            password_required: room.password.is_some(),
        }))
    } else {
        // TODO(r.floren) Do we want to return something else here?
        Err(ApiError::not_found())
    }
}
