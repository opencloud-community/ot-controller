// SPDX-FileCopyrightText: OpenTalk GmbH <mail@opentalk.eu>
//
// SPDX-License-Identifier: EUPL-1.2

//! Response types for REST APIv1
//!
//! These all implement the [`Responder`] trait.
use actix_web::{HttpResponse, Responder, body::BoxBody};
use opentalk_types_api_v1::error::ApiError;

pub mod error;
mod ok;

pub use ok::ApiResponse;

/// The default API Result
pub type DefaultApiResult<T, E = ApiError> = Result<ApiResponse<T>, E>;

/// Represents a 201 Created HTTP Response
pub struct Created;

impl Responder for Created {
    type Body = BoxBody;

    fn respond_to(self, _: &actix_web::HttpRequest) -> HttpResponse {
        HttpResponse::Created().finish()
    }
}

/// Represents a 204 No Content HTTP Response
pub struct NoContent;

impl Responder for NoContent {
    type Body = BoxBody;

    fn respond_to(self, _: &actix_web::HttpRequest) -> HttpResponse {
        HttpResponse::NoContent().finish()
    }
}

/// Represents a 304 Not Modified HTTP Response
pub struct NotModified;

impl Responder for NotModified {
    type Body = BoxBody;

    fn respond_to(self, _: &actix_web::HttpRequest) -> HttpResponse {
        HttpResponse::NotModified().finish()
    }
}
