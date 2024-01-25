// SPDX-FileCopyrightText: OpenTalk GmbH <mail@opentalk.eu>
//
// SPDX-License-Identifier: EUPL-1.2

use displaydoc::Display;
use opentalk_database::DatabaseError;
use opentalk_types::api::error::ApiError;
use thiserror::Error;

/// Errors returned when deleting an event
#[derive(Display, Error, Debug)]
pub enum Error {
    /// Error from the database
    Database(#[from] DatabaseError),

    /// Error from the permissions system (kustos)
    Kustos(#[from] kustos::Error),

    /// Tried to perform an action that is forbidden for the user
    Forbidden,

    /// Object deletion error: {0}
    ObjectDeletion(#[source] anyhow::Error),

    /// Shared folders not configured
    SharedFoldersNotConfigured,

    /// Nextcloud client error: {0}
    NextcloudClient(#[from] opentalk_nextcloud_client::Error),

    /// Error: {0}
    Custom(String),
}

impl From<Error> for ApiError {
    fn from(value: Error) -> Self {
        match value {
            Error::Database(e) => Self::from(e),
            Error::Kustos(e) => Self::from(e),
            Error::Forbidden => Self::forbidden(),
            Error::ObjectDeletion(e) => Self::from(e),
            Error::SharedFoldersNotConfigured => {
                Self::bad_request().with_message("No shared folder configured for this server")
            }
            Error::NextcloudClient(_e) => {
                Self::internal().with_message("Error performing actions on the NextCloud")
            }
            Error::Custom(e) => Self::internal().with_message(e.to_string()),
        }
    }
}
