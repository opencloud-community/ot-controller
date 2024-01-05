// SPDX-FileCopyrightText: OpenTalk GmbH <mail@opentalk.eu>
//
// SPDX-License-Identifier: EUPL-1.2

use std::num::ParseIntError;

use thiserror::Error;

/// The error type returned when parsing invalid values from strings.
///
/// Derived using [`thiserror::Error`]
#[derive(Debug, Error)]
pub enum ParsingError {
    #[error("Invalid access method: `{0}`")]
    InvalidAccessMethod(String),
    #[error("String was not a PolicyUser casbin string: `{0}`")]
    PolicyUser(String),
    #[error("String was not a PolicyInvite casbin string: `{0}`")]
    PolicyInvite(String),
    #[error("String was not a PolicyInternalGroup casbin string: `{0}")]
    PolicyInternalGroup(String),
    #[error("String was not a PolicyOPGroup casbin string: `{0}`")]
    PolicyOPGroup(String),
    #[error("Failed to parse UUID")]
    Uuid(#[from] uuid::Error),
    #[error("Custom: {0}")]
    Custom(String),
}

/// The error returned when a resource failed to be parsed
///
/// Currently supported types are only uuids and integers, all other use the fallback Other variant.
#[derive(Debug, Error)]
pub enum ResourceParseError {
    #[error("Failed to parse UUID")]
    Uuid(#[from] uuid::Error),
    #[error(transparent)]
    ParseInt(#[from] ParseIntError),
    #[error(transparent)]
    Other(#[from] anyhow::Error),
}
