// SPDX-FileCopyrightText: OpenTalk GmbH <mail@opentalk.eu>
//
// SPDX-License-Identifier: EUPL-1.2

use std::error::Error;

use url::Url;

use crate::ApiError;

/// A trait representing a client which can communicate with an OpenTalk instance via REST.
pub trait RestClient {
    /// The errors which may occur for this client.
    type Error: Error + Send + Sync + 'static;

    /// Get the URL for the endpoint for the client.
    ///
    /// This method adds the base url including hostname for the client's target instance.
    ///
    /// # Errors
    ///
    /// The error that can be returned by the method will usually indicate that
    /// parsing the generated URL string into a [Url] failed.
    fn rest_endpoint(&self, endpoint: &str) -> Result<Url, ApiError<Self::Error>>;
}
