// SPDX-FileCopyrightText: OpenTalk GmbH <mail@opentalk.eu>
//
// SPDX-License-Identifier: EUPL-1.2

//! Error response types for REST APIv1
use actix_http::header::HeaderValue;
use actix_web::error::JsonPayloadError;
use actix_web::http::StatusCode;
use actix_web::HttpRequest;
use opentalk_types::api::error::{ApiError, ErrorBody};
use serde::Deserialize;
use serde::Serialize;

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
    www_authenticate: Option<Vec<u8>>,
    body: ErrorBody,
}

impl TryFrom<CacheableApiError> for ApiError {
    type Error = anyhow::Error;

    fn try_from(value: CacheableApiError) -> Result<Self, Self::Error> {
        Ok(ApiError {
            status: StatusCode::from_u16(value.status)?,
            www_authenticate: value
                .www_authenticate
                .map(|b| HeaderValue::from_bytes(&b))
                .transpose()?,
            body: value.body,
        })
    }
}

impl From<&ApiError> for CacheableApiError {
    fn from(value: &ApiError) -> Self {
        Self {
            status: value.status.as_u16(),
            www_authenticate: value
                .www_authenticate
                .as_ref()
                .map(|h| h.as_bytes().to_owned()),
            body: value.body.clone(),
        }
    }
}
