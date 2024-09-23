// SPDX-FileCopyrightText: OpenTalk GmbH <mail@opentalk.eu>
//
// SPDX-License-Identifier: EUPL-1.2

use http_request_derive::HttpRequest;
use opentalk_types_api_v1::auth::GetLoginResponseBody;

/// *GET* request on `/auth/login`
#[derive(Clone, Debug, PartialEq, Eq, Hash, HttpRequest)]
#[http_request(
        method = "GET",
        response = GetLoginResponseBody,
        path = "/v1/auth/login",
)]
pub struct GetLoginRequest;
