// SPDX-FileCopyrightText: OpenTalk GmbH <mail@opentalk.eu>
//
// SPDX-License-Identifier: EUPL-1.2

#[allow(unused_imports)]
use crate::imports::*;

use super::PostInviteVerifyRequestBody;

/// Verify body for *POST /invite/verify*
#[derive(Clone, Debug, PartialEq, Eq)]
#[cfg_attr(
    feature = "frontend",
    derive(HttpRequest),
    http_request(method = "POST", response = super::CodeVerified, path = "/v1/invite/verify")
)]
pub struct PostInviteVerifyRequest(
    #[cfg_attr(feature = "frontend", http_request(body))] pub PostInviteVerifyRequestBody,
);
