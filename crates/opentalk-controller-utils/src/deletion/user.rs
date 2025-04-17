// SPDX-FileCopyrightText: OpenTalk GmbH <mail@opentalk.eu>
//
// SPDX-License-Identifier: EUPL-1.2

//! Functionality to delete users including all associated resources

use diesel_async::{scoped_futures::ScopedFutureExt, AsyncConnection};
use kustos::Authz;
use log::Log;
use opentalk_controller_settings::SettingsRaw;
use opentalk_database::{DatabaseError, DbConnection};
use opentalk_db_storage::{groups::remove_user_from_all_groups, users::User};
use opentalk_log::debug;
use opentalk_signaling_core::{ExchangeHandle, ObjectStorage};
use opentalk_types_common::users::UserId;

use super::{Deleter, Error};
/// Delete a user by id including the corresponding room and resources it
/// references.
#[derive(Debug)]
pub struct UserDeleter {
    user_id: UserId,
}

impl UserDeleter {
    /// Create a new `UserDeleter`.
    pub fn new(user_id: UserId) -> Self {
        Self { user_id }
    }
}

#[async_trait::async_trait]
impl Deleter for UserDeleter {
    type PreparedCommit = ();
    type CommitOutput = ();

    async fn prepare_commit(
        &self,
        _logger: &dyn Log,
        _conn: &mut DbConnection,
    ) -> Result<Self::PreparedCommit, Error> {
        Ok(())
    }

    async fn check_permissions(
        &self,
        _prepared_commit: &Self::PreparedCommit,
        _logger: &dyn Log,
        _authz: &Authz,
        _user_id: Option<UserId>,
    ) -> Result<(), Error> {
        Ok(())
    }

    async fn pre_commit(
        &self,
        _prepared_commit: &Self::PreparedCommit,
        _logger: &dyn Log,
        _conn: &mut DbConnection,
        _exchange_handle: ExchangeHandle,
        _settings: &SettingsRaw,
    ) -> Result<(), Error> {
        Ok(())
    }

    async fn commit_to_database(
        &self,
        _prepared_commit: Self::PreparedCommit,
        logger: &dyn Log,
        conn: &mut DbConnection,
    ) -> Result<Self::CommitOutput, Error> {
        let user_id = self.user_id;

        debug!(log: logger, "Deleting all database resources of user {user_id}");
        let _transaction_result: Result<(), DatabaseError> = conn
            .transaction(|conn| {
                async move {
                    remove_user_from_all_groups(conn, user_id).await?;
                    User::delete_by_id(conn, user_id).await?;

                    Ok(())
                }
                .scope_boxed()
            })
            .await;

        Ok(())
    }

    async fn post_commit(
        &self,
        _commit_output: (),
        _logger: &dyn Log,
        _settings: &SettingsRaw,
        authz: &Authz,
        _storage: &ObjectStorage,
    ) -> Result<(), Error> {
        let _ = authz.remove_all_user_groups_and_roles(self.user_id).await?;

        Ok(())
    }
}
