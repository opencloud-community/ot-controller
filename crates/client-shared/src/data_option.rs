// SPDX-FileCopyrightText: OpenTalk GmbH <mail@opentalk.eu>
//
// SPDX-License-Identifier: EUPL-1.2

use std::error::Error;

use bytes::Bytes;
use http::StatusCode;
use serde::de::DeserializeOwned;

use crate::{ApiError, FromHttpResponse};

/// A wrapper around [`Option`] for deserializing from optional data from a HTTP response.
///
/// The [`Option`] must be wrapped in order to implement [`FromHttpResponse`], because the
/// default implementation for [`Option`] where the data is [`DeserializeOwned`] would conflict
/// with the direct implementation for types that are [`DeserializeOwned`] themselves.
#[derive(Debug, derive_more::From, derive_more::Into)]
pub struct DataOption<T>(pub Option<T>);

impl<D: DeserializeOwned> FromHttpResponse for DataOption<D> {
    fn from_http_response<E>(http_response: http::Response<Bytes>) -> Result<Self, ApiError<E>>
    where
        E: Error + Send + Sync + 'static,
        Self: Sized,
    {
        if http_response.status() == StatusCode::NO_CONTENT {
            return Ok(None.into());
        };
        let data = serde_json::from_slice(http_response.body())?;
        Ok(Some(data).into())
    }
}
