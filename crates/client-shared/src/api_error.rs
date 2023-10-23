// SPDX-FileCopyrightText: OpenTalk GmbH <mail@opentalk.eu>
// SPDX-FileCopyrightText: Kitware, Inc
//
// SPDX-License-Identifier: EUPL-1.2

use std::error::Error;

use bytes::Bytes;
use thiserror::Error;

/// Errors which may occur when using API endpoints.
#[derive(Debug, Error)]
#[non_exhaustive]
pub enum ApiError<E>
where
    E: Error + Send + Sync + 'static,
{
    /// The client encountered an error.
    #[error("client error: {}", source)]
    Client {
        /// The client error.
        source: E,
    },

    /// The URL failed to parse.
    #[error("failed to parse url: {}", source)]
    UrlParse {
        /// The source of the error.
        #[from]
        source: url::ParseError,
    },

    /// The URI failed to parse.
    #[error("failed to parse uri: {}", source)]
    UriParse {
        /// The source of the error.
        #[from]
        source: http::uri::InvalidUri,
    },

    /// JSON deserialization from OpenTalk failed.
    #[error("could not parse JSON response: {}", source)]
    Json {
        /// The source of the error.
        #[from]
        source: serde_json::Error,
    },

    /// OpenTalk returned an error message.
    #[error("opentalk server error: {}", msg)]
    OpenTalk {
        /// The error message from OpenTalk.
        msg: String,
    },

    /// OpenTalk returned an error without JSON information.
    #[error("opentalk internal server error {}", status)]
    OpenTalkService {
        /// The status code for the return.
        status: http::StatusCode,
        /// The error data from OpenTalk.
        data: Bytes,
    },

    /// Failed to parse an expected data type from JSON.
    #[error("could not parse {} data from JSON: {}", typename, source)]
    DataType {
        /// The source of the error.
        source: serde_json::Error,
        /// The name of the type that could not be deserialized.
        typename: &'static str,
    },

    /// Couldn't build a HTTP request, probably a bug.
    #[error("could not build HTTP request: {}", source)]
    Request {
        /// The source of the error
        source: http::Error,
    },

    /// Trying to perform an unauthorized request
    #[error("trying to perfom an unauthorized request")]
    Unauthorized,
}
