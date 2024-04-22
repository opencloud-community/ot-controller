// SPDX-FileCopyrightText: OpenTalk GmbH <mail@opentalk.eu>
//
// SPDX-License-Identifier: EUPL-1.2

use std::borrow::Cow;

use utoipa::{ToResponse, ToSchema};

/// Internal reusable dummy type for utoipa internal server error
#[derive(ToResponse)]
#[response(description = "An internal server error occurred")]
pub struct InternalServerError;

/// Internal reusable dummy type for utoipa unauthorized error
#[derive(ToResponse)]
#[response(
    description = "The provided access token is expired or the provided id or access token is invalid.
        The WWW-Authenticate header will contain an error description 'session expired' to distinguish between
        an invalid and an expired token",
    headers(
        (
            "www-authenticate",
            description = r#"
Will contain 'session expired' to distinguish between an invalid and an expired token.

Examples:

    Bearer error="invalid_token", error_description="The provided access token is invalid"
    Bearer error="invalid_request", error_description="The user session expired"
"#
        ),
    ),
)]
pub struct Unauthorized {
    /// Machine readable error code
    pub code: Cow<'static, str>,

    /// Human readable message
    pub message: Cow<'static, str>,
}

/// Internal reusable dummy type for utoipa forbidden error
#[derive(ToResponse)]
#[response(description = "The authorized user has no permission to access the requested resource")]
pub struct Forbidden;

#[derive(ToResponse)]
#[response(description = "Bad request")]
pub struct BadRequest;

#[derive(ToResponse, ToSchema)]
#[response(description = "Binary data", content_type = "application/octet-stream")]
#[schema(value_type = String, format = KnownFormat::Binary)]
pub struct BinaryData;
