// SPDX-FileCopyrightText: OpenTalk GmbH <mail@opentalk.eu>
//
// SPDX-License-Identifier: EUPL-1.2

//! Types and traits that are used by the OpenTalk client library crate

#![warn(
    bad_style,
    missing_debug_implementations,
    missing_docs,
    overflowing_literals,
    patterns_in_fns_without_body,
    trivial_casts,
    trivial_numeric_casts,
    unsafe_code,
    unused,
    unused_extern_crates,
    unused_import_braces,
    unused_qualifications,
    unused_results,
    clippy::pedantic
)]

use serde::{de::DeserializeOwned, Serialize};

/// Access methods used by client for accessing the API
#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum Method {
    /// HTTP GET method
    GET,

    /// HTTP POST method
    POST,

    /// HTTP PUT method
    PUT,

    /// HTTP PATCH method
    PATCH,

    /// HTTP DELETE method
    DELETE,
}

/// A trait implemented for types that are sent to the API as parameters
pub trait Request: std::fmt::Debug + Serialize {
    /// The response type that is expected to the request
    type Response: DeserializeOwned;

    /// The API endpoint path relative to the base URL
    const PATH: &'static str;

    /// The method used to send the data to the API endpoint
    const METHOD: Method;
}
