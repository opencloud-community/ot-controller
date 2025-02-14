// SPDX-FileCopyrightText: OpenTalk GmbH <mail@opentalk.eu>
//
// SPDX-License-Identifier: EUPL-1.2

use std::{future::Future, pin::Pin};

use openidconnect::{reqwest::Error, HttpRequest, HttpResponse};

pub fn make_client() -> Result<reqwest11::Client, reqwest11::Error> {
    reqwest11::Client::builder()
        .redirect(reqwest11::redirect::Policy::none())
        .build()
}

pub type BoxedHttpResponseFuture =
    Box<dyn Future<Output = Result<HttpResponse, Error<reqwest11::Error>>>>;

pub fn async_http_client(
    client: reqwest11::Client,
) -> impl Fn(HttpRequest) -> Pin<BoxedHttpResponseFuture> {
    move |request| Box::pin(async_http_client_inner(client.clone(), request))
}

async fn async_http_client_inner(
    client: reqwest11::Client,
    request: HttpRequest,
) -> Result<HttpResponse, Error<reqwest11::Error>> {
    let mut request_builder = client
        .request(request.method, request.url.as_str())
        .body(request.body);
    for (name, value) in &request.headers {
        request_builder = request_builder.header(name.as_str(), value.as_bytes());
    }
    let request = request_builder.build().map_err(Error::Reqwest)?;

    let response = client.execute(request).await.map_err(Error::Reqwest)?;

    let status_code = response.status();
    let headers = response.headers().to_owned();
    let chunks = response.bytes().await.map_err(Error::Reqwest)?;
    Ok(HttpResponse {
        status_code,
        headers,
        body: chunks.to_vec(),
    })
}
