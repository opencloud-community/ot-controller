// SPDX-FileCopyrightText: OpenTalk GmbH <mail@opentalk.eu>
//
// SPDX-License-Identifier: EUPL-1.2

use shared::error::{ParsingError, ResourceParseError};
use thiserror::Error;
use tokio::task::JoinError;

/// A combining error type which is returned by most major kustos methods
///
/// Derived using [`thiserror::Error`]
#[derive(Debug, Error)]
pub enum Error {
    #[error("Not Authorized")]
    NotAuthorized,
    #[error("Failed to convert string to type, {0}")]
    ParsingError(#[from] ParsingError),
    #[error("Failed to convert Resource to type, {0}")]
    ResourceParsingError(#[from] ResourceParseError),
    #[error("Blocking error, {0}")]
    BlockingError(#[from] JoinError),
    #[error("Casbin error {0}")]
    CasbinError(#[from] casbin::Error),
    #[error("Tried to start already running authz enforcer autoload")]
    AutoloadRunning,
    #[error("{0}")]
    Custom(String),
}

/// A default specialized Result type for kustos
pub type Result<T, E = Error> = std::result::Result<T, E>;
