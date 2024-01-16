// SPDX-FileCopyrightText: OpenTalk GmbH <mail@opentalk.eu>
//
// SPDX-License-Identifier: EUPL-1.2

use crate::{Authorization, AuthorizedRequest, Request};

/// Trait for adding authorization information to http requests
pub trait WithAuthorization: Request + Sized {
    /// Augment the request with authorization information
    fn with_authorization<A: Authorization>(self, authorization: A) -> AuthorizedRequest<A, Self> {
        AuthorizedRequest::new(authorization, self)
    }
}

impl<R: Request + Sized> WithAuthorization for R {}
