// SPDX-FileCopyrightText: OpenTalk GmbH <mail@opentalk.eu>
//
// SPDX-License-Identifier: EUPL-1.2

use bytes::Bytes;
use http::StatusCode;
use http_request_derive::FromHttpResponse;
use serde::de::DeserializeOwned;

/// A wrapper around [`Option`] for deserializing from optional data from a HTTP response.
///
/// The [`Option`] must be wrapped in order to implement [`FromHttpResponse`], because the
/// default implementation for [`Option`] where the data is [`DeserializeOwned`] would conflict
/// with the direct implementation for types that are [`DeserializeOwned`] themselves.
#[derive(Debug, derive_more::From, derive_more::Into)]
pub struct DataOption<T>(pub Option<T>);

impl<D: DeserializeOwned> FromHttpResponse for DataOption<D> {
    fn from_http_response(
        http_response: http::Response<Bytes>,
    ) -> Result<Self, http_request_derive::Error>
    where
        Self: Sized,
    {
        if http_response.status() == StatusCode::NO_CONTENT {
            return Ok(None.into());
        };
        let data = serde_json::from_slice(http_response.body()).map_err(|e| {
            http_request_derive::Error::custom(format!("Couldn't deserialize JSON: {e:?}"))
        })?;
        Ok(Some(data).into())
    }
}
