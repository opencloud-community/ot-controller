// SPDX-FileCopyrightText: OpenTalk GmbH <mail@opentalk.eu>
//
// SPDX-License-Identifier: EUPL-1.2

//! This module contains types that are used for OpenTalk API V1 auth endpoints.

use std::collections::HashSet;

#[allow(unused_imports)]
use crate::imports::*;

/// Body of the response to a *POST* request on `/auth/login`
#[derive(Clone, Debug, PartialEq, Eq)]
#[cfg_attr(feature = "serde", derive(Serialize, Deserialize))]
#[cfg_attr(feature = "utoipa", derive(utoipa::ToSchema))]
pub struct PostLoginResponse {
    /// Permissions is a set of strings that each define a permission a user has.
    pub permissions: HashSet<String>,
}

/// *GET* request on `/auth/login`
#[derive(Clone, Debug, PartialEq, Eq, Hash)]
#[cfg_attr(
    feature = "frontend",
    derive(HttpRequest),
    http_request(
        method = "GET",
        response = opentalk_types_api_v1::auth::GetLoginResponseBody,
        path = "/v1/auth/login",
    ),
)]
pub struct GetLoginRequest;
