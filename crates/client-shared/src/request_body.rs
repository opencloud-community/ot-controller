// SPDX-FileCopyrightText: OpenTalk GmbH <mail@opentalk.eu>
//
// SPDX-License-Identifier: EUPL-1.2

use std::error::Error;

use http::HeaderMap;
use serde::Serialize;

use crate::ApiError;

/// Trait defined on every body type that can be used with [`crate::Request`].
pub trait RequestBody {
    /// Convert the request contents to a [`Vec<u8>`].
    ///
    /// # Errors
    ///
    /// If the conversion to the [`Vec`] goes wrong, this will be indicated by
    /// returning an appropriate [`ApiError`].
    fn to_vec<E>(&self) -> Result<Vec<u8>, ApiError<E>>
    where
        E: Error + Send + Sync + 'static;

    /// Apply the headers that this body requires for the request to be valid.
    /// This will usually add a [`http::header::CONTENT_TYPE`] header.
    fn apply_headers(&self, _headers: &mut HeaderMap) {}
}

impl<B: Serialize> RequestBody for B {
    fn to_vec<E>(&self) -> Result<Vec<u8>, ApiError<E>>
    where
        E: Error + Send + Sync + 'static,
    {
        serde_json::to_vec(self).map_err(|source| ApiError::Json { source })
    }

    fn apply_headers(&self, headers: &mut HeaderMap) {
        let _ = headers
            .entry(http::header::CONTENT_TYPE)
            .or_insert_with(|| http::HeaderValue::from_static("application/json"));
    }
}
