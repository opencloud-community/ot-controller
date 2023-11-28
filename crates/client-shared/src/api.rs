// SPDX-FileCopyrightText: OpenTalk GmbH <mail@opentalk.eu>
// SPDX-FileCopyrightText: Kitware, Inc
//
// SPDX-License-Identifier: EUPL-1.2

use async_trait::async_trait;

mod error;

pub use error::ApiError;

/// A trait which represents an asynchronous query which may be made to an OpenTalk client.
#[async_trait]
pub trait Query<T, C>
where
    C: Client,
{
    /// Perform the query asynchronously against the client.
    async fn query(&self, client: &C) -> Result<T, ApiError<C::Error>>;
}
