// SPDX-FileCopyrightText: OpenTalk GmbH <mail@opentalk.eu>
//
// SPDX-License-Identifier: EUPL-1.2

use http::{header::CONTENT_TYPE, Uri};
use serde::Serialize;

use crate::{ApiError, RestClient};

use super::Request;

/// A trait that is implemented on types that can be sent as HTTP requests through a [`RestClient`].
pub trait ToHttpRequest: Request {
    /// Convert the type to a [`http::request::Request`] for use with a specific [`RestClient`].
    ///
    /// # Errors
    ///
    /// An error will be returned whenever the conversion fails, e.g. because serialization is not possible.
    fn to_http_request<C: RestClient>(
        &self,
        c: &C,
    ) -> Result<http::request::Request<Vec<u8>>, ApiError<C::Error>>;
}

impl<R: Serialize + Request> ToHttpRequest for R {
    fn to_http_request<C: RestClient>(
        &self,
        c: &C,
    ) -> Result<http::request::Request<Vec<u8>>, ApiError<C::Error>> {
        let uri = c.rest_endpoint(&self.path())?.as_str().parse::<Uri>()?;
        let body = serde_json::to_vec(&self)?;

        http::request::Request::builder()
            .method(Self::METHOD)
            .uri(uri)
            .header(CONTENT_TYPE, "application/json")
            .body(body)
            .map_err(|source| ApiError::Request { source })
    }
}
