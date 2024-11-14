// SPDX-FileCopyrightText: OpenTalk GmbH <mail@opentalk.eu>
//
// SPDX-License-Identifier: EUPL-1.2

use http_request_derive::HttpRequest;

use crate::{Authorization, AuthorizedHttpRequest};

/// Trait for adding authorization information to http requests
pub trait WithAuthorization: HttpRequest + Sized {
    /// Augment the request with authorization information
    fn with_authorization<A: Authorization>(
        self,
        authorization: A,
    ) -> AuthorizedHttpRequest<A, Self> {
        AuthorizedHttpRequest::new(authorization, self)
    }
}

impl<R: HttpRequest + Sized> WithAuthorization for R {}
