// SPDX-FileCopyrightText: OpenTalk GmbH <mail@opentalk.eu>
//
// SPDX-License-Identifier: EUPL-1.2

use database::DatabaseError;
use displaydoc::Display;
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
    NextcloudClient(#[from] nextcloud_client::Error),

    /// Error: {0}
    Custom(String),
}
