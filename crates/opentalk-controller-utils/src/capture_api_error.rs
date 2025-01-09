// SPDX-FileCopyrightText: OpenTalk GmbH <mail@opentalk.eu>
//
// SPDX-License-Identifier: EUPL-1.2

use std::fmt::Display;

use opentalk_database::DatabaseError;
use opentalk_signaling_core::{assets::AssetError, ObjectStorageError};
use opentalk_types_api_v1::error::ApiError;
use snafu::Whatever;

/// Wraps an [`ApiError`]. Allows converting foreign error types that are not
/// visible to the [`ApiError`] by using the `?` operator.
#[derive(Debug, Clone, PartialEq, Eq)]
pub struct CaptureApiError(pub ApiError);

impl From<CaptureApiError> for ApiError {
    fn from(CaptureApiError(error): CaptureApiError) -> Self {
        error
    }
}

impl From<ApiError> for CaptureApiError {
    fn from(value: ApiError) -> Self {
        CaptureApiError(value)
    }
}

impl<'a> From<&'a CaptureApiError> for &'a ApiError {
    fn from(value: &'a CaptureApiError) -> Self {
        &value.0
    }
}

impl From<DatabaseError> for CaptureApiError {
    fn from(value: DatabaseError) -> Self {
        log::error!(
            "REST API threw internal error from Diesel error: {}",
            snafu::Report::from_error(value)
        );
        CaptureApiError(ApiError::internal())
    }
}

impl From<diesel::result::Error> for CaptureApiError {
    fn from(value: diesel::result::Error) -> Self {
        Self::from(<diesel::result::Error as Into<DatabaseError>>::into(value))
    }
}

impl From<kustos::Error> for CaptureApiError {
    fn from(value: kustos::Error) -> Self {
        log::error!("REST API threw internal error from kustos error: {value}");
        CaptureApiError(ApiError::internal())
    }
}

impl From<ObjectStorageError> for CaptureApiError {
    fn from(value: ObjectStorageError) -> Self {
        log::error!("REST API threw internal error from object storage error: {value}");
        CaptureApiError(ApiError::internal())
    }
}

impl From<opentalk_cache::CacheError> for CaptureApiError {
    fn from(value: opentalk_cache::CacheError) -> Self {
        log::error!("REST API threw internal error while writing/reading cache: {value}");
        CaptureApiError(ApiError::internal())
    }
}

impl From<Whatever> for CaptureApiError {
    fn from(value: Whatever) -> Self {
        let error_report = snafu::Report::from_error(value);
        log::error!("REST API threw generic internal error: {error_report:?}");
        CaptureApiError(ApiError::internal())
    }
}

impl From<AssetError> for CaptureApiError {
    fn from(value: AssetError) -> Self {
        log::error!("REST API threw internal error: {value:?}");
        CaptureApiError(ApiError::internal())
    }
}

impl Display for CaptureApiError {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        self.0.fmt(f)
    }
}

impl actix_web::ResponseError for CaptureApiError {
    fn status_code(&self) -> actix_web::http::StatusCode {
        self.0.status_code()
    }

    fn error_response(&self) -> actix_web::HttpResponse<actix_web::body::BoxBody> {
        self.0.error_response()
    }
}
