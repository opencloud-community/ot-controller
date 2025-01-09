// SPDX-FileCopyrightText: OpenTalk GmbH <mail@opentalk.eu>
//
// SPDX-License-Identifier: EUPL-1.2

use kustos_shared::{resource::ResourceParseError, subject::ParsingError};
use snafu::Snafu;
use tokio::task::JoinError;

/// A combining error type which is returned by most major kustos methods
///
/// Derived using [`snafu::Snafu`]
#[derive(Debug, Snafu)]
pub enum Error {
    #[snafu(display("Failed to convert string to type, {source}"), context(false))]
    ParsingError { source: ParsingError },

    #[snafu(
        display("Failed to convert Resource to type, {source}"),
        context(false)
    )]
    ResourceParsingError { source: ResourceParseError },

    #[snafu(display("Blocking error, {source}"), context(false))]
    BlockingError { source: JoinError },

    #[snafu(display("Casbin error {source}"), context(false))]
    CasbinError { source: casbin::Error },

    #[snafu(display("Tried to start already running authz enforcer autoload"))]
    AutoloadRunning,

    #[snafu(whatever)]
    Custom { message: String },
}

/// A default specialized Result type for kustos
pub type Result<T, E = Error> = std::result::Result<T, E>;
