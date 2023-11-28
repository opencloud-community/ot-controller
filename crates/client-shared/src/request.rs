// SPDX-FileCopyrightText: OpenTalk GmbH <mail@opentalk.eu>
//
// SPDX-License-Identifier: EUPL-1.2

pub(crate) mod authorized;
pub(crate) mod from_http_response;
pub(crate) mod to_http_request;
pub(crate) mod with_authorization;

use std::error::Error;

use bytes::Bytes;
use http::StatusCode;
use serde::Serialize;

use crate::{ApiError, FromHttpResponse};

/// A trait implemented for types that are sent to the API as parameters
pub trait Request: std::fmt::Debug {
    /// The response type that is expected to the request
    type Response: FromHttpResponse;

    /// The method used to send the data to the API endpoint
    const METHOD: http::Method;

    /// Get the API endpoint path relative to the base URL
    fn path(&self) -> String;

    /// Get query parameters for the `http::Request`
    fn query<T: Serialize + Sized>(&self) -> Option<T> {
        None
    }

    /// Convert the response from a `http::Response`
    ///
    /// # Errors
    ///
    /// Usually HTTP response codes that don't indicate success will be converted to the
    /// corresponding [`ApiError`]. For example, a [`StatusCode::UNAUTHORIZED`] is converted
    /// to [`ApiError::Unauthorized`]. This is the behavior found in the default implementation
    /// and can be overwritten by a specialized implementation if required.
    fn read_response<E>(response: http::Response<Bytes>) -> Result<Self::Response, ApiError<E>>
    where
        E: Error + Send + Sync + 'static,
    {
        match response.status() {
            status if status.is_success() => Self::Response::from_http_response(response),

            StatusCode::UNAUTHORIZED => Err(ApiError::Unauthorized),
            status => Err(ApiError::OpenTalkService {
                status,
                data: response.into_body(),
            }),
        }
    }
}
