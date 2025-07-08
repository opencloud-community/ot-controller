// SPDX-FileCopyrightText: OpenTalk GmbH <mail@opentalk.eu>
//
// SPDX-License-Identifier: EUPL-1.2

use bigdecimal::BigDecimal;
use opentalk_db_storage::{
    groups::{
        Group, insert_user_into_groups, remove_user_from_all_groups, remove_user_from_groups,
    },
    users::{NewUser, UpdateUser, User},
};
use opentalk_inventory::{UserInventory, error::StorageBackendSnafu};
use opentalk_types_common::{tenants::TenantId, time::Timestamp, users::UserId};
use snafu::ResultExt as _;

use crate::{DatabaseConnection, Result};

#[async_trait::async_trait]
impl UserInventory for DatabaseConnection {
    #[tracing::instrument(err, skip_all)]
    async fn create_user(&mut self, new_user: NewUser) -> Result<User> {
        new_user
            .insert(&mut self.inner)
            .await
            .context(StorageBackendSnafu)
    }

    #[tracing::instrument(err, skip_all)]
    async fn get_user(&mut self, user_id: UserId) -> Result<User> {
        User::get(&mut self.inner, user_id)
            .await
            .context(StorageBackendSnafu)
    }

    #[tracing::instrument(err, skip_all)]
    async fn update_user<'a>(&mut self, user_id: UserId, user: UpdateUser<'a>) -> Result<User> {
        user.apply(&mut self.inner, user_id)
            .await
            .context(StorageBackendSnafu)
    }

    #[tracing::instrument(err, skip_all)]
    async fn delete_user(&mut self, user_id: UserId) -> Result<()> {
        User::delete_by_id(&mut self.inner, user_id)
            .await
            .context(StorageBackendSnafu)
    }

    #[tracing::instrument(err, skip_all)]
    async fn get_all_users(&mut self) -> Result<Vec<User>> {
        User::get_all(&mut self.inner)
            .await
            .context(StorageBackendSnafu)
    }

    #[tracing::instrument(err, skip_all)]
    async fn get_users_by_ids(&mut self, user_ids: &[UserId]) -> Result<Vec<User>> {
        User::get_all_by_ids(&mut self.inner, user_ids)
            .await
            .context(StorageBackendSnafu)
    }

    #[tracing::instrument(err, skip_all)]
    async fn get_user_ids_disabled_before(&mut self, timestamp: Timestamp) -> Result<Vec<UserId>> {
        User::get_disabled_before(&mut self.inner, timestamp.into())
            .await
            .context(StorageBackendSnafu)
    }

    #[tracing::instrument(err, skip_all)]
    async fn get_user_by_email(
        &mut self,
        tenant_id: TenantId,
        email_address: &str,
    ) -> Result<Option<User>> {
        User::get_by_email(&mut self.inner, tenant_id, email_address)
            .await
            .context(StorageBackendSnafu)
    }

    #[tracing::instrument(err, skip_all)]
    async fn get_users_by_phone_number(
        &mut self,
        tenant_id: TenantId,
        phone_number_e164: &str,
    ) -> Result<Vec<User>> {
        User::get_by_phone(&mut self.inner, tenant_id, phone_number_e164)
            .await
            .context(StorageBackendSnafu)
    }

    #[tracing::instrument(err, skip_all)]
    async fn get_user_by_odic_sub(
        &mut self,
        tenant_id: TenantId,
        sub: &str,
    ) -> Result<Option<User>> {
        User::get_by_oidc_sub(&mut self.inner, tenant_id, sub)
            .await
            .context(StorageBackendSnafu)
    }

    #[tracing::instrument(err, skip_all)]
    async fn get_users_by_odic_subs(
        &mut self,
        tenant_id: TenantId,
        subs: &[&str],
    ) -> Result<Vec<User>> {
        User::get_all_by_oidc_subs(&mut self.inner, tenant_id, subs)
            .await
            .context(StorageBackendSnafu)
    }

    #[tracing::instrument(err, skip_all)]
    async fn get_user_for_tenant(&mut self, tenant_id: TenantId, user_id: UserId) -> Result<User> {
        User::get_filtered_by_tenant(&mut self.inner, tenant_id, user_id)
            .await
            .context(StorageBackendSnafu)
    }

    #[tracing::instrument(err, skip_all)]
    async fn add_user_to_groups(&mut self, user: &User, groups: &[Group]) -> Result<()> {
        insert_user_into_groups(&mut self.inner, user, groups)
            .await
            .context(StorageBackendSnafu)
    }

    #[tracing::instrument(err, skip_all)]
    async fn remove_user_from_groups(&mut self, user: &User, groups: &[Group]) -> Result<()> {
        remove_user_from_groups(&mut self.inner, user, groups)
            .await
            .context(StorageBackendSnafu)
    }

    #[tracing::instrument(err, skip_all)]
    async fn remove_user_from_all_groups(&mut self, user_id: UserId) -> Result<()> {
        remove_user_from_all_groups(&mut self.inner, user_id)
            .await
            .context(StorageBackendSnafu)
    }

    #[tracing::instrument(err, skip_all)]
    async fn get_user_storage_used_size(&mut self, user_id: UserId) -> Result<BigDecimal> {
        User::get_used_storage(&mut self.inner, &user_id)
            .await
            .context(StorageBackendSnafu)
    }

    #[tracing::instrument(err, skip_all)]
    async fn get_user_storage_used_size_u64(&mut self, user_id: UserId) -> Result<u64> {
        User::get_used_storage_u64(&mut self.inner, &user_id)
            .await
            .context(StorageBackendSnafu)
    }

    #[tracing::instrument(err, skip_all)]
    async fn find_users(
        &mut self,
        tenant_id: TenantId,
        search_string: &str,
        limit: usize,
    ) -> Result<Vec<User>> {
        User::find(&mut self.inner, tenant_id, search_string, limit)
            .await
            .context(StorageBackendSnafu)
    }
}
