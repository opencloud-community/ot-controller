// SPDX-FileCopyrightText: OpenTalk GmbH <mail@opentalk.eu>
//
// SPDX-License-Identifier: EUPL-1.2

//! This module contains types that are used for OpenTalk API V1 auth endpoints.

use std::collections::HashSet;

#[allow(unused_imports)]
use crate::imports::*;

/// A POST request to the `/auth/login` endpoint
#[derive(Clone, Debug, PartialEq, Eq)]
#[cfg_attr(
    feature = "frontend",
    derive(HttpRequest),
    http_request(
        method = "POST",
        response = PostLoginResponse,
        path = "/v1/auth/login"
    )
)]
pub struct PostLoginRequest(
    #[cfg_attr(feature = "frontend", http_request(body))] PostLoginRequestBody,
);

/// Body of a *POST* request on `/auth/login`
#[derive(Clone, Debug, PartialEq, Eq)]
#[cfg_attr(feature = "serde", derive(Serialize, Deserialize))]
pub struct PostLoginRequestBody {
    /// The id token to use for the login
    pub id_token: String,
}

/// Body of the response to a *POST* request on `/auth/login`
#[derive(Clone, Debug, PartialEq, Eq)]
#[cfg_attr(feature = "serde", derive(Serialize, Deserialize))]
pub struct PostLoginResponse {
    /// Permissions is a set of strings that each define a permission a user has.
    pub permissions: HashSet<String>,
}

/// *GET* request on `/auth/login`
#[derive(Clone, Debug, PartialEq, Eq, Hash)]
#[cfg_attr(
    feature = "frontend",
    derive(HttpRequest),
    http_request(method = "GET", response = GetLoginResponse, path = "/v1/auth/login")
)]
#[cfg_attr(feature = "serde", derive(Serialize, Deserialize))]
pub struct GetLoginRequest;

/// Body of the response to a *GET* request on `/auth/login`
#[derive(Clone, Debug, PartialEq, Eq, Hash)]
#[cfg_attr(feature = "serde", derive(Serialize, Deserialize))]
pub struct GetLoginResponse {
    /// Description of the OIDC provider to use for the login
    pub oidc: OidcProvider,
}

/// Represents an OIDC provider
#[derive(Clone, Debug, PartialEq, Eq, Hash)]
#[cfg_attr(feature = "serde", derive(Serialize, Deserialize))]
pub struct OidcProvider {
    /// The name of the provider
    pub name: String,

    /// The url of the provider
    pub url: String,
}
