// SPDX-FileCopyrightText: OpenTalk GmbH <mail@opentalk.eu>
// SPDX-FileCopyrightText: Kitware, Inc
//
// SPDX-License-Identifier: EUPL-1.2

use std::error::Error;

use bytes::Bytes;
use snafu::prelude::*;

/// Errors which may occur when using API endpoints.
#[derive(Debug, Snafu)]
#[snafu(visibility(pub))]
#[non_exhaustive]
pub enum ApiError<E>
where
    E: Error + Send + Sync + 'static,
{
    /// The client encountered an error.
    #[snafu(display("client error: {}", source))]
    Client {
        /// The source of the error.
        source: E,
    },

    /// The URL failed to parse.
    #[snafu(context(false), display("failed to parse url: {}", source))]
    UrlParse {
        /// The source of the error.
        source: url::ParseError,
    },

    /// The URI failed to parse.
    #[snafu(context(false), display("failed to parse uri: {}", source))]
    UriParse {
        /// The source of the error.
        source: http::uri::InvalidUri,
    },

    /// JSON deserialization from OpenTalk failed.
    #[snafu(context(false), display("could not parse JSON response: {}", source))]
    Json {
        /// The source of the error.
        source: serde_json::Error,
    },

    /// OpenTalk returned an error message.
    #[snafu(display("opentalk server error: {}", msg))]
    OpenTalk {
        /// The error message from OpenTalk.
        msg: String,
    },

    /// OpenTalk returned an error without JSON information.
    #[snafu(display("opentalk internal server error {}", status))]
    OpenTalkService {
        /// The status code for the return.
        status: http::StatusCode,
        /// The error data from OpenTalk.
        data: Bytes,
    },

    /// Failed to parse an expected data type from JSON.
    #[snafu(display("could not parse {} data from JSON: {}", typename, source))]
    DataType {
        /// The source of the error.
        source: serde_json::Error,
        /// The name of the type that could not be deserialized.
        typename: &'static str,
    },

    /// Error from the http-request-derive crate.
    #[snafu(
        context(false),
        display("error in http-request-derive crate: {}", source)
    )]
    HttpRequestDerive {
        /// The source of the error.
        source: http_request_derive::Error,
    },

    /// Couldn't build a HTTP request, probably a bug.
    #[snafu(context(false), display("could not build HTTP request: {}", source))]
    Request {
        /// The source of the error.
        source: http::Error,
    },

    /// Trying to perform an unauthorized request.
    #[snafu(display("trying to perfom an unauthorized request"))]
    Unauthorized,

    /// Custom error
    #[snafu(whatever)]
    Custom {
        /// The custom error message
        message: String,

        /// The source of the error.
        #[snafu(source(from(Box<dyn Error + Send + Sync>, Some)))]
        source: Option<Box<dyn Error + Send + Sync>>,
    },
}
