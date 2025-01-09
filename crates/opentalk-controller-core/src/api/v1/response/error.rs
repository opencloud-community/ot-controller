// SPDX-FileCopyrightText: OpenTalk GmbH <mail@opentalk.eu>
//
// SPDX-License-Identifier: EUPL-1.2

//! Error response types for REST APIv1
use actix_web::{error::JsonPayloadError, HttpRequest};
use http::StatusCode;
use opentalk_controller_utils::CaptureApiError;
use opentalk_types_api_v1::error::{ApiError, AuthenticationError, ErrorBody};
use serde::{Deserialize, Serialize};
use snafu::ResultExt;

/// Error handler for the actix JSON extractor
///
/// Gets called when a incoming request results in an [`JsonPayloadError`].
/// Returns a `Bad Request` [`ApiError`] error with an appropriate error code and message.
pub fn json_error_handler(err: JsonPayloadError, _: &HttpRequest) -> actix_web::error::Error {
    let error_code = match err {
        JsonPayloadError::OverflowKnownLength { .. } | JsonPayloadError::Overflow { .. } => {
            "payload_overflow"
        }
        JsonPayloadError::ContentType => "invalid_content_type",
        JsonPayloadError::Deserialize(_) | JsonPayloadError::Serialize(_) => "invalid_json",
        _ => "invalid_payload",
    };
    ApiError::bad_request()
        .with_code(error_code)
        .with_message(err.to_string())
        .into()
}

/// (De)Serializable version of [`ApiError`] so it can be externally cached
#[derive(Clone, Serialize, Deserialize)]
pub struct CacheableApiError {
    status: u16,
    www_authenticate: Option<AuthenticationError>,
    body: ErrorBody,
}

impl TryFrom<CacheableApiError> for CaptureApiError {
    type Error = crate::Whatever;

    fn try_from(value: CacheableApiError) -> Result<Self, Self::Error> {
        Ok(ApiError {
            status: StatusCode::from_u16(value.status).whatever_context("Invalid status code")?,
            www_authenticate: value.www_authenticate,
            body: value.body,
        }
        .into())
    }
}

impl From<&ApiError> for CacheableApiError {
    fn from(value: &ApiError) -> Self {
        Self {
            status: value.status.as_u16(),
            www_authenticate: value.www_authenticate,
            body: value.body.clone(),
        }
    }
}

impl From<&CaptureApiError> for CacheableApiError {
    fn from(value: &CaptureApiError) -> Self {
        Self::from(&value.0)
    }
}
