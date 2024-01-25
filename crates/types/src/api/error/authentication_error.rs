// SPDX-FileCopyrightText: OpenTalk GmbH <mail@opentalk.eu>
//
// SPDX-License-Identifier: EUPL-1.2

/// Error variants for the WWW Authenticate header
#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum AuthenticationError {
    /// The provided id token is invalid
    InvalidIdToken,

    /// The provided access token is invalid
    InvalidAccessToken,

    /// The provided access token expired
    AccessTokenInactive,

    /// The user session expired"
    SessionExpired,
}

impl AuthenticationError {
    /// Get the error message for the error
    pub const fn message(&self) -> &'static str {
        match self {
            Self::InvalidIdToken => "The provided id token is invalid",
            Self::InvalidAccessToken => "The provided access token is invalid",
            Self::AccessTokenInactive => "The provided access token expired",
            Self::SessionExpired => "The user session expired",
        }
    }
}

#[cfg(feature = "actix")]
impl From<AuthenticationError> for actix_web_httpauth::extractors::bearer::Error {
    fn from(value: AuthenticationError) -> Self {
        use actix_web_httpauth::extractors::bearer::Error;

        match value {
            AuthenticationError::InvalidIdToken
            | AuthenticationError::InvalidAccessToken
            | AuthenticationError::AccessTokenInactive => Error::InvalidToken,
            AuthenticationError::SessionExpired => Error::InvalidRequest,
        }
    }
}
