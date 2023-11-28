// SPDX-FileCopyrightText: OpenTalk GmbH <mail@opentalk.eu>
//
// SPDX-License-Identifier: EUPL-1.2

use crate::{Authorization, Request, ToHttpRequest};

/// Wrapper type that adds authorization information to a request
#[derive(Debug)]
pub struct Authorized<A: Authorization, R: ToHttpRequest> {
    authorization: A,
    request: R,
}

impl<A: Authorization, R: ToHttpRequest> Authorized<A, R> {
    /// Create a new authorized request
    pub const fn new(authorization: A, request: R) -> Self {
        Self {
            authorization,
            request,
        }
    }
}

impl<A: Authorization, R: ToHttpRequest> Request for Authorized<A, R> {
    type Response = R::Response;

    const METHOD: http::Method = R::METHOD;

    fn path(&self) -> String {
        self.request.path()
    }

    fn query<T: serde::Serialize + Sized>(&self) -> Option<T> {
        self.request.query()
    }

    fn read_response<E>(
        response: http::Response<bytes::Bytes>,
    ) -> Result<Self::Response, crate::ApiError<E>>
    where
        E: std::error::Error + Send + Sync + 'static,
    {
        R::read_response(response)
    }
}

impl<A: Authorization, R: ToHttpRequest> ToHttpRequest for Authorized<A, R> {
    fn to_http_request<C: crate::RestClient>(
        &self,
        c: &C,
    ) -> Result<http::request::Request<Vec<u8>>, crate::ApiError<C::Error>> {
        let mut request = self.request.to_http_request(c)?;

        self.authorization
            .add_authorization_information(&mut request);

        Ok(request)
    }
}
