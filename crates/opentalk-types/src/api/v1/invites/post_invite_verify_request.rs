// SPDX-FileCopyrightText: OpenTalk GmbH <mail@opentalk.eu>
//
// SPDX-License-Identifier: EUPL-1.2

use opentalk_types_api_v1::rooms::by_room_id::invites::PostInviteVerifyRequestBody;

#[allow(unused_imports)]
use crate::imports::*;

/// Verify body for *POST /invite/verify*
#[derive(Clone, Debug, PartialEq, Eq)]
#[cfg_attr(
    feature = "frontend",
    derive(HttpRequest),
    http_request(
        method = "POST",
        response = opentalk_types_api_v1::rooms::by_room_id::invites::PostInviteVerifyResponseBody,
        path = "/v1/invite/verify"
    )
)]
pub struct PostInviteVerifyRequest(
    #[cfg_attr(feature = "frontend", http_request(body))] pub PostInviteVerifyRequestBody,
);
