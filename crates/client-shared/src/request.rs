// SPDX-FileCopyrightText: OpenTalk GmbH <mail@opentalk.eu>
//
// SPDX-License-Identifier: EUPL-1.2

pub(crate) mod to_http_request;

/// A trait implemented for types that are sent to the API as parameters
pub trait Request: std::fmt::Debug {
    /// The response type that is expected to the request
    type Response;

    /// The API endpoint path relative to the base URL
    const PATH: &'static str;

    /// The method used to send the data to the API endpoint
    const METHOD: http::Method;
}
