// SPDX-FileCopyrightText: OpenTalk GmbH <mail@opentalk.eu>
//
// SPDX-License-Identifier: EUPL-1.2

use opentalk_database::DatabaseError;
use opentalk_signaling_core::ObjectStorageError;
use opentalk_types::api::error::ApiError;
use snafu::Snafu;

/// Errors returned when deleting an event
#[derive(Debug, Snafu)]
#[snafu(visibility(pub(crate)))]
pub enum Error {
    /// Database error
    #[snafu(display("Database error: {source}"), context(false))]
    Database {
        /// the cause of the error
        source: DatabaseError,
    },

    /// Kustos error
    #[snafu(display("Authorization error (kustos): {source}"), context(false))]
    Kustos {
        /// the cause of the error
        source: kustos::Error,
    },

    /// Forbidden action by user
    #[snafu(display("Tried to perform an action that is forbidden for the user"))]
    Forbidden,

    /// Conflict error
    #[snafu(display(
        "Conflict error: The requested operation could not be completed due to a conflict: {message}"
    ))]
    Conflict {
        /// Error message
        message: String,
    },

    /// Object deletion error
    #[snafu(display("Object deletion error: {source}"))]
    ObjectDeletion {
        /// the cause of the error
        source: ObjectStorageError,
    },

    /// Shared folders not configured
    #[snafu(display("Shared folders not configured"))]
    SharedFoldersNotConfigured,

    /// Nextcloud client error
    #[snafu(display("Nextcloud client error: {source}"), context(false))]
    NextcloudClient {
        /// the cause of the error
        source: opentalk_nextcloud_client::Error,
    },

    /// Custom error
    #[snafu(display("{message}: "), whatever)]
    Custom {
        /// Error message
        message: String,
        /// the cause of the error
        #[snafu(source(from(Box<dyn std::error::Error + Sync + Send>, Some)))]
        source: Option<Box<dyn std::error::Error + Sync + Send>>,
    },
}

impl From<Error> for ApiError {
    fn from(value: Error) -> Self {
        match value {
            Error::Database { source } => Self::from(source),
            Error::Kustos { source } => Self::from(source),
            Error::Forbidden => Self::forbidden(),
            Error::Conflict { message } => Self::conflict().with_message(message),
            Error::ObjectDeletion { source } => {
                log::error!("REST API threw internal error from object storage: {source}");
                ApiError::internal()
            }
            Error::SharedFoldersNotConfigured => {
                Self::bad_request().with_message("No shared folder configured for this server")
            }
            Error::NextcloudClient { .. } => {
                Self::internal().with_message("Error performing actions on the NextCloud")
            }
            Error::Custom { message, source: _ } => Self::internal().with_message(message),
        }
    }
}
