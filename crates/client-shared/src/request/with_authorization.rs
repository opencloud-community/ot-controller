// SPDX-FileCopyrightText: OpenTalk GmbH <mail@opentalk.eu>
//
// SPDX-License-Identifier: EUPL-1.2

use crate::{Authorization, AuthorizedRequest, ToHttpRequest};

/// Trait for adding authorization information to http requests
pub trait WithAuthorization: ToHttpRequest + Sized {
    /// Augment the request with authorization information
    fn with_authorization<A: Authorization>(self, authorization: A) -> AuthorizedRequest<A, Self> {
        AuthorizedRequest::new(authorization, self)
    }
}

impl<R: ToHttpRequest + Sized> WithAuthorization for R {}
