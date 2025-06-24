// SPDX-FileCopyrightText: OpenTalk GmbH <mail@opentalk.eu>
//
// SPDX-License-Identifier: EUPL-1.2

use bigdecimal::BigDecimal;
use opentalk_db_storage::{
    groups::Group,
    users::{NewUser, UpdateUser, User},
};
use opentalk_types_common::{tenants::TenantId, time::Timestamp, users::UserId};

use crate::Result;

/// A trait for retrieving and storing user entities.
#[async_trait::async_trait]
pub trait UserInventory {
    /// Create a new user.
    async fn create_user(&mut self, new_user: NewUser) -> Result<User>;

    /// Get a user by its id.
    async fn get_user(&mut self, user_id: UserId) -> Result<User>;

    /// Update a user.
    async fn update_user<'a>(&mut self, user_id: UserId, user: UpdateUser<'a>) -> Result<User>;

    /// Delete a user.
    async fn delete_user(&mut self, user_id: UserId) -> Result<()>;

    /// Get all users.
    async fn get_all_users(&mut self) -> Result<Vec<User>>;

    /// Get a list of users by their ids.
    async fn get_users_by_ids(&mut self, user_ids: &[UserId]) -> Result<Vec<User>>;

    /// Get a list of users that have been disabled before a certain timestamp.
    async fn get_user_ids_disabled_before(&mut self, timestamp: Timestamp) -> Result<Vec<UserId>>;

    /// Get a user by their E-Mail address
    async fn get_user_by_email(
        &mut self,
        tenant_id: TenantId,
        email_address: &str,
    ) -> Result<Option<User>>;

    /// Get users by their phone number (in E.164 format).
    async fn get_users_by_phone_number(
        &mut self,
        tenant_id: TenantId,
        phone_number_e164: &str,
    ) -> Result<Vec<User>>;

    /// Get a user by the value in their OIDC `sub` field.
    async fn get_user_by_odic_sub(
        &mut self,
        tenant_id: TenantId,
        sub: &str,
    ) -> Result<Option<User>>;

    /// Get users by the values in their OIDC `sub` fields.
    async fn get_users_by_odic_subs(
        &mut self,
        tenant_id: TenantId,
        subs: &[&str],
    ) -> Result<Vec<User>>;

    /// Get a specific user inside a tenant.
    ///
    /// If no user with that id exists in that tenant, an error will be returned.
    async fn get_user_for_tenant(&mut self, tenant_id: TenantId, user_id: UserId) -> Result<User>;

    /// Add a user to one or multiple groups.
    async fn add_user_to_groups(&mut self, user: &User, groups: &[Group]) -> Result<()>;

    /// Remove a user from one or multiple groups.
    async fn remove_user_from_groups(&mut self, user: &User, groups: &[Group]) -> Result<()>;

    /// Remove a user from all groups.
    async fn remove_user_from_all_groups(&mut self, user_id: UserId) -> Result<()>;

    /// Get the storage used by a user.
    async fn get_user_storage_used_size(&mut self, user_id: UserId) -> Result<BigDecimal>;

    /// Get the storage used by a user.
    async fn get_user_storage_used_size_u64(&mut self, user_id: UserId) -> Result<u64>;

    /// Find users by a search string.
    ///
    /// Considers the display_name, firstname, lastname and email fields.
    async fn find_users(
        &mut self,
        tenant_id: TenantId,
        search_string: &str,
        limit: usize,
    ) -> Result<Vec<User>>;
}
