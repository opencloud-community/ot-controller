// SPDX-FileCopyrightText: OpenTalk GmbH <mail@opentalk.eu>
//
// SPDX-License-Identifier: EUPL-1.2

use std::error::Error;

use bytes::Bytes;
use serde::de::DeserializeOwned;

use crate::ApiError;

/// Trait for types that can be converted from `http::Response`
pub trait FromHttpResponse {
    /// Convert from `http::Response` to our `Response`
    ///
    /// # Errors
    ///
    /// The implementation of this trait will map the response to an error if it should be interpreted as such.
    /// Typical HTTP status code errors are read by the default implementation of [`crate::Request::read_response`]
    /// already, so in most cases additional checks are not necessary here.
    ///
    /// Of course if the contents of the response cannot be parsed, this will usually be handled as an
    /// error as well.
    fn from_http_response<E>(http_response: http::Response<Bytes>) -> Result<Self, ApiError<E>>
    where
        E: Error + Send + Sync + 'static,
        Self: Sized;
}

impl<D: DeserializeOwned> FromHttpResponse for D {
    fn from_http_response<E>(http_response: http::Response<Bytes>) -> Result<Self, ApiError<E>>
    where
        E: Error + Send + Sync + 'static,
    {
        serde_json::from_slice(http_response.body()).map_err(Into::into)
    }
}
