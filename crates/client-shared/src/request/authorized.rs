// SPDX-FileCopyrightText: OpenTalk GmbH <mail@opentalk.eu>
//
// SPDX-License-Identifier: EUPL-1.2

use http::StatusCode;
use http_request_derive::HttpRequest;

use crate::Authorization;

/// Wrapper type that adds authorization information to a request
#[derive(Debug)]
pub struct Authorized<A: Authorization, R: HttpRequest> {
    authorization: A,
    request: R,
}

impl<A: Authorization, R: HttpRequest> Authorized<A, R> {
    /// Create a new authorized request
    pub const fn new(authorization: A, request: R) -> Self {
        Self {
            authorization,
            request,
        }
    }
}

impl<A: Authorization, R: HttpRequest> HttpRequest for Authorized<A, R> {
    type Response = R::Response;

    type Query = R::Query;

    type Body = R::Body;

    const METHOD: http::Method = R::METHOD;

    fn path(&self) -> String {
        self.request.path()
    }

    fn query(&self) -> Option<&Self::Query> {
        self.request.query()
    }

    fn body(&self) -> Option<&Self::Body> {
        self.request.body()
    }

    fn apply_headers(&self, headers: &mut http::HeaderMap) {
        self.request.apply_headers(headers);
        self.authorization.apply_authorization_headers(headers);
    }

    fn read_response(
        response: http::Response<bytes::Bytes>,
    ) -> Result<Self::Response, http_request_derive::Error> {
        match response.status() {
            StatusCode::UNAUTHORIZED => Err(http_request_derive::Error::Unauthorized),
            _ => R::read_response(response),
        }
    }
}
