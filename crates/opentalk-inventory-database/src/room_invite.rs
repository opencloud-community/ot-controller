// SPDX-FileCopyrightText: OpenTalk GmbH <mail@opentalk.eu>
//
// SPDX-License-Identifier: EUPL-1.2

use opentalk_db_storage::invites::{Invite, InviteWithUsers, NewInvite, UpdateInvite};
use opentalk_inventory::{RoomInviteInventory, error::StorageBackendSnafu};
use opentalk_types_common::{
    rooms::{RoomId, invite_codes::InviteCode},
    time::Timestamp,
    users::UserId,
};
use snafu::ResultExt as _;

use crate::{DatabaseConnection, Result};

#[async_trait::async_trait]
impl RoomInviteInventory for DatabaseConnection {
    #[tracing::instrument(err, skip_all)]
    async fn create_room_invite(&mut self, invite: NewInvite) -> Result<Invite> {
        invite
            .insert(&mut self.inner)
            .await
            .context(StorageBackendSnafu)
    }

    #[tracing::instrument(err, skip_all)]
    async fn get_room_invite(&mut self, invite_code: InviteCode) -> Result<Invite> {
        Invite::get(&mut self.inner, invite_code)
            .await
            .context(StorageBackendSnafu)
    }

    #[tracing::instrument(err, skip_all)]
    async fn get_first_invite_for_room(&mut self, room_id: RoomId) -> Result<Option<Invite>> {
        Invite::get_first_for_room(&mut self.inner, room_id)
            .await
            .context(StorageBackendSnafu)
    }

    #[tracing::instrument(err, skip_all)]
    async fn get_or_create_first_invite_for_room(
        &mut self,
        room_id: RoomId,
        user_id: UserId,
    ) -> Result<Invite> {
        Invite::get_first_or_create_for_room(&mut self.inner, room_id, user_id)
            .await
            .context(StorageBackendSnafu)
    }

    #[tracing::instrument(err, skip_all)]
    async fn get_room_invites_updated_by(&mut self, user_id: UserId) -> Result<Vec<Invite>> {
        Invite::get_updated_by(&mut self.inner, user_id)
            .await
            .context(StorageBackendSnafu)
    }

    #[tracing::instrument(err, skip_all)]
    async fn get_room_invites_paginated_with_creator_and_updater(
        &mut self,
        room_id: RoomId,
        limit: i64,
        page: i64,
    ) -> Result<(Vec<InviteWithUsers>, i64)> {
        Invite::get_all_for_room_with_users_paginated(&mut self.inner, room_id, limit, page)
            .await
            .context(StorageBackendSnafu)
    }

    #[tracing::instrument(err, skip_all)]
    async fn get_room_invite_with_creator_and_updater(
        &mut self,
        invite_code: InviteCode,
    ) -> Result<InviteWithUsers> {
        Invite::get_with_users(&mut self.inner, invite_code)
            .await
            .context(StorageBackendSnafu)
    }

    #[tracing::instrument(err, skip_all)]
    async fn update_room_invite(
        &mut self,
        room_id: RoomId,
        invite_code: InviteCode,
        invite: UpdateInvite,
    ) -> Result<Invite> {
        invite
            .apply(&mut self.inner, room_id, invite_code)
            .await
            .context(StorageBackendSnafu)
    }

    #[tracing::instrument(err, skip_all)]
    async fn get_room_invites_with_room_inactive_or_expired_before(
        &mut self,
        expired_before: Timestamp,
    ) -> Result<Vec<(InviteCode, RoomId)>> {
        Invite::get_inactive_or_expired_before(&mut self.inner, expired_before.into())
            .await
            .context(StorageBackendSnafu)
    }
}
