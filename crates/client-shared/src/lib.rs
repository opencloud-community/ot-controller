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

mod api_error;
mod client;
mod data_option;
mod request;

pub use api_error::ApiError;
pub use client::{rest_client::RestClient, Client};
pub use data_option::DataOption;
pub use request::{from_http_response::FromHttpResponse, to_http_request::ToHttpRequest, Request};
