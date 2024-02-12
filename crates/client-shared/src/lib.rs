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
mod authorization;
mod client;
mod data_option;
mod request;
mod request_body;

pub use api_error::ApiError;
pub use authorization::Authorization;
pub use client::{rest_client::RestClient, Client};
pub use data_option::DataOption;
pub use request::{
    authorized::Authorized as AuthorizedRequest, from_http_response::FromHttpResponse,
    with_authorization::WithAuthorization, Request,
};
pub use request_body::RequestBody;

/// Re-export of the `Request` proc-macro
///
/// Which can be derived by structs that needs to implement the `Request` trait
/// It generates the request types (e.g. Response, Body or Query) and the getter functions
///
/// # Examples:
///
/// ## GET Endpoint with dynamic path
///
/// #[derive(Request)]
/// #[request(
///     method = "GET",
///     response = "`GetItemResponse`",
///     path = "/v1/item/{0}"
/// )]
/// pub struct `GetItemRequest`(pub Id);
///
/// ## GET Endpoint with query parameters
///
/// #[derive(Request)]
/// #[request(
///     method = "GET",
///     response = "`NoContent`",
///     path = "/v1/items"
/// )]
/// pub struct `GetItemsWithQueryRequest` {
///     #[request(query)]
///     pub query: `PostEventInviteQuery`,
/// }
///
/// ## POST Endpoint with multiple attributes
///
/// #[derive(Request)]
/// #[request(
///     method = "POST",
///     response = "`SampleResponse`",
///     path = "/v1/item/{id}"
/// )]
/// pub struct `SampleRequest` {
///     pub id: Id,
///     #[request(query)]
///     pub query: `SampleQuery`,
///     #[request(body)]
///     pub body: `SampleBody`,
///     #[request(header)]
///     pub header: `http::HeaderMap`,
/// }
pub use opentalk_client_shared_impl::Request;

#[doc(hidden)]
pub mod __exports {
    pub use http;
}
