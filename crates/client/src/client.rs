// SPDX-FileCopyrightText: OpenTalk GmbH <mail@opentalk.eu>
//
// SPDX-License-Identifier: EUPL-1.2

use shared::{ApiError, RestClient, ToHttpRequest};
use url::Url;

pub struct Client {
    client: reqwest::Client,
    base_url: Url,
}

impl Client {
    pub fn new(base_url: Url) -> Self {
        let client = reqwest::Client::new();

        Self { client, base_url }
    }
}

#[async_trait::async_trait]
impl RestClient for Client {
    type Error = reqwest::Error;

    fn rest_endpoint(&self, endpoint: &str) -> Result<Url, ApiError<Self::Error>> {
        self.base_url.join(endpoint).map_err(Into::into)
    }
}

#[async_trait::async_trait]
impl shared::Client for Client {
    async fn rest<R: ToHttpRequest + Send>(
        &self,
        request: R,
    ) -> Result<R::Response, ApiError<Self::Error>> {
        let request = request.to_http_request(self)?;
        let request =
            reqwest::Request::try_from(request).map_err(|source| ApiError::Client { source })?;
        let response = self
            .client
            .execute(request)
            .await
            .map_err(|source| ApiError::Client { source })?;
        let mut http_response = http::Response::builder()
            .status(response.status())
            .version(response.version());
        if let Some(headers) = http_response.headers_mut() {
            *headers = response.headers().clone();
        }
        let body = response
            .bytes()
            .await
            .map_err(|source| ApiError::Client { source })?;
        let http_response = http_response
            .body(body)
            .map_err(|source| ApiError::Request { source })?;
        R::read_response::<Self::Error>(http_response)
    }
}
