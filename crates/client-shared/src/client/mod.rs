// SPDX-FileCopyrightText: OpenTalk GmbH <mail@opentalk.eu>
// SPDX-FileCopyrightText: Kitware, Inc
//
// SPDX-License-Identifier: EUPL-1.2

use async_trait::async_trait;
use http_request_derive::HttpRequest;

use self::rest_client::RestClient;
use crate::ApiError;

pub(crate) mod rest_client;

/// A trait representing an asynchronous client which can communicate with an OpenTalk instance.
#[async_trait]
pub trait Client: RestClient {
    /// Send a REST query asynchronously.
    async fn rest<R: HttpRequest + Send>(
        &self,
        request: R,
    ) -> Result<R::Response, ApiError<Self::Error>>;
}
