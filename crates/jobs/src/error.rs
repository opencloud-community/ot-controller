// SPDX-FileCopyrightText: OpenTalk GmbH <mail@opentalk.eu>
//
// SPDX-License-Identifier: EUPL-1.2

use opentalk_signaling_core::ObjectStorageError;
use snafu::Snafu;

/// Errors that can occur during job execution
#[derive(Debug, Snafu)]
#[snafu(visibility(pub(crate)))]
pub enum Error {
    /// Could not load job parameters
    ParameterLoading {
        /// The error source
        source: serde_json::Error,
    },

    /// Could not serialize job parameters
    ParameterSerializing {
        /// The error source
        source: serde_json::Error,
    },

    /// Job execution timed out
    #[snafu(context(false))]
    Timeout {
        /// The error source
        source: tokio::time::error::Elapsed,
    },

    /// Database error
    #[snafu(context(false))]
    Database {
        /// The error source
        source: opentalk_database::DatabaseError,
    },

    /// Job execution failed
    JobExecutionFailed,

    /// Permission system error
    #[snafu(context(false))]
    Kustos {
        /// The error source
        source: kustos::Error,
    },

    /// Found shared folders in the database, but the configuration file contains no shared folder settings
    SharedFoldersNotConfigured,

    /// Error communicating with NextCloud instance: {source}
    #[snafu(context(false))]
    NextcloudClient {
        /// The error source
        source: opentalk_nextcloud_client::Error,
    },

    /// Error performing changes in the object storage
    #[snafu(context(false))]
    ObjectStorage {
        /// The error source
        source: ObjectStorageError,
    },

    /// Event deletion failed
    EventDeletionFailed,
}
