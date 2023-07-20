// SPDX-FileCopyrightText: OpenTalk GmbH <mail@opentalk.eu>
//
// SPDX-License-Identifier: EUPL-1.2

use displaydoc::Display;

/// Errors that can occur during job execution
#[derive(Debug, Display, thiserror::Error)]
pub enum Error {
    /// Could not load job parameters
    ParameterLoading(#[from] serde_json::Error),

    /// Job execution timed out
    Timeout(#[from] tokio::time::error::Elapsed),

    /// Database error
    Database(#[from] database::DatabaseError),

    /// Job execution failed
    JobExecutionFailed,

    /// Permission system error
    Kustos(#[from] kustos::Error),

    /// Found shared folders in the database, but the configuration file contains no shared folder settings
    SharedFoldersNotConfigured,

    /// Error communicating with NextCloud instance: {0:?}
    NextcloudClient(#[from] nextcloud_client::Error),

    /// Error performing changes in the object storage
    ObjectStorage(#[source] anyhow::Error),

    /// {0}
    Custom(String),

    /// Event deletion failed: {0:?}
    EventDeletionFailed(#[from] controller_utils::deletion::Error),
}
