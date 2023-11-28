// SPDX-FileCopyrightText: OpenTalk GmbH <mail@opentalk.eu>
//
// SPDX-License-Identifier: EUPL-1.2

//! This module contains types that are used for OpenTalk API V1 auth endpoints.

use std::collections::HashSet;

#[allow(unused_imports)]
use crate::imports::*;

#[cfg(feature = "frontend")]
const LOGIN_PATH: &str = "/v1/auth/login";

/// Body of a *POST* request on `/auth/login`
#[derive(Clone, Debug, PartialEq, Eq)]
#[cfg_attr(feature = "serde", derive(Serialize, Deserialize))]
pub struct PostLoginRequest {
    /// The id token to use for the login
    pub id_token: String,
}

#[cfg(feature = "frontend")]
impl Request for PostLoginRequest {
    type Response = PostLoginResponse;
    const METHOD: Method = Method::POST;

    fn path(&self) -> String {
        LOGIN_PATH.into()
    }
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
#[cfg_attr(feature = "serde", derive(Serialize, Deserialize))]
pub struct GetLoginRequest;

#[cfg(feature = "frontend")]
impl Request for GetLoginRequest {
    type Response = GetLoginResponse;
    const METHOD: Method = Method::GET;

    fn path(&self) -> String {
        LOGIN_PATH.into()
    }
}

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
