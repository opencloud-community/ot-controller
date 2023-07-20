// SPDX-FileCopyrightText: OpenTalk GmbH <mail@opentalk.eu>
//
// SPDX-License-Identifier: EUPL-1.2

use database::DbConnection;
use kustos::Authz;
use log::Log;
use settings::Settings;
use signaling_core::{ExchangeHandle, ObjectStorage};
use types::core::UserId;

use super::Error;

/// A trait for performing multi-stage deletion of database elements according
/// to this sequence:
/// - Collect referenced datasets that might be changed by others during the process
/// - Check permissions
/// - Perform pre-commit actions, such as removing referenced data from external systems
/// - Commit the changes to the database in a transaction, with a race condition protection
/// - Perform post-commit actions, similar to the pre-commit actions
#[async_trait::async_trait]
pub trait Deleter: Sync {
    /// The outcome of the database commit preparation.
    type PreparedCommit: Sync + Send;

    /// The outcome of the database commit.
    type CommitOutput: Sync + Send;

    /// Perform a full sequence of the steps implemented by trait.
    #[allow(clippy::too_many_arguments)]
    async fn perform(
        &self,
        logger: &dyn Log,
        conn: &mut DbConnection,
        authz: &Authz,
        user_id: Option<UserId>,
        exchange_handle: ExchangeHandle,
        settings: &Settings,
        object_storage: &ObjectStorage,
    ) -> Result<(), Error> {
        let prepared_commit = self.prepare_commit(logger, conn).await?;
        self.check_permissions(&prepared_commit, logger, authz, user_id)
            .await?;
        self.pre_commit(&prepared_commit, logger, conn, exchange_handle, settings)
            .await?;
        let commit_output = self
            .commit_to_database(prepared_commit, logger, conn)
            .await?;
        self.post_commit(commit_output, logger, settings, authz, object_storage)
            .await?;
        Ok(())
    }

    /// Prepare the database commit.
    ///
    /// This should be used to collect data that can be used in the subsequent
    /// steps of the deletion sequence. That can be a list of referenced items,
    /// a list of permissions to be checked in the `check_permissions` step or
    /// other relevant information.
    async fn prepare_commit(
        &self,
        logger: &dyn Log,
        conn: &mut DbConnection,
    ) -> Result<Self::PreparedCommit, Error>;

    /// Check the permissions that are required to perform the deletion.
    ///
    /// This function should return an `Err` if the permissions are not sufficient.
    /// The `user_id` parameter either contains the user who wants to perform the
    /// action (e.g. when called through an API endpoint), or `None` if this
    /// action runs outside the permission system (e.g. by command-line tooling
    /// executed by an administrator).
    async fn check_permissions(
        &self,
        prepared_commit: &Self::PreparedCommit,
        logger: &dyn Log,
        authz: &Authz,
        user_id: Option<UserId>,
    ) -> Result<(), Error>;

    /// Execute actions before the database commit is performed.
    async fn pre_commit(
        &self,
        _prepared_commit: &Self::PreparedCommit,
        _logger: &dyn Log,
        _conn: &mut DbConnection,
        _exchange_handle: ExchangeHandle,
        _settings: &Settings,
    ) -> Result<(), Error> {
        Ok(())
    }

    /// Commit the changes to the database.
    async fn commit_to_database(
        &self,
        prepared_commit: Self::PreparedCommit,
        logger: &dyn Log,
        conn: &mut DbConnection,
    ) -> Result<Self::CommitOutput, Error>;

    /// Execute actions after the database commit was performed.
    async fn post_commit(
        &self,
        _commit_output: Self::CommitOutput,
        _logger: &dyn Log,
        _settings: &Settings,
        _authz: &Authz,
        _storage: &ObjectStorage,
    ) -> Result<(), Error> {
        Ok(())
    }
}
