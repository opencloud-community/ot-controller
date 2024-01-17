// SPDX-FileCopyrightText: OpenTalk GmbH <mail@opentalk.eu>
//
// SPDX-License-Identifier: EUPL-1.2

pub(crate) mod authorized;
pub(crate) mod from_http_response;
pub(crate) mod with_authorization;

use std::error::Error;

use bytes::Bytes;
use http::{HeaderMap, StatusCode, Uri};
use serde::Serialize;

use crate::{ApiError, FromHttpResponse, RequestBody, RestClient};

/// A trait implemented for types that are sent to the API as parameters
pub trait Request: std::fmt::Debug {
    /// The response type that is expected to the request
    type Response: FromHttpResponse;

    /// The query type that is sent to the API endpoint
    type Query: Serialize;

    /// The body type that is sent to the API endpoint
    type Body: RequestBody;

    /// The method used to send the data to the API endpoint
    const METHOD: http::Method;

    /// Get the API endpoint path relative to the base URL
    fn path(&self) -> String;

    /// Get query parameters for the `http::Request`
    fn query(&self) -> Option<&Self::Query> {
        None
    }

    /// Get the body for the `http::Request`
    fn body(&self) -> Option<&Self::Body> {
        None
    }

    /// Get the headers for the `http::Request`
    fn apply_headers(&self, headers: &mut HeaderMap) {
        let _ = headers
            .entry(http::header::CONTENT_TYPE)
            .or_insert_with(|| http::HeaderValue::from_static("application/json"));
    }

    /// Build a HTTP request from the request type
    ///
    /// # Errors
    /// The
    fn to_http_request<C: RestClient>(
        &self,
        c: &C,
    ) -> Result<http::request::Request<Vec<u8>>, ApiError<C::Error>> {
        let body = self
            .body()
            .map(RequestBody::to_vec)
            .transpose()?
            .unwrap_or_default();

        let uri = {
            let uri = c.rest_endpoint(&self.path())?.as_str().parse::<Uri>()?;

            if let Some(query) = self.query() {
                let query = serde_url_params::to_string(query)
                    .expect("couldn't serialize query parameters");
                format!("{uri}?{query}",).parse::<Uri>()?
            } else {
                uri
            }
        };

        let mut headers = HeaderMap::new();
        if let Some(body) = self.body() {
            body.apply_headers(&mut headers);
        }

        let mut builder = http::request::Request::builder()
            .method(Self::METHOD)
            .uri(uri);

        for (name, value) in &headers {
            builder = builder.header(name, value);
        }

        builder
            .body(body)
            .map_err(|source| ApiError::Request { source })
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
