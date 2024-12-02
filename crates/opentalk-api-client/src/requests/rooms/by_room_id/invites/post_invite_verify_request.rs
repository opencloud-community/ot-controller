// SPDX-FileCopyrightText: OpenTalk GmbH <mail@opentalk.eu>
//
// SPDX-License-Identifier: EUPL-1.2

use http_request_derive::HttpRequest;
use opentalk_types_api_v1::rooms::by_room_id::invites::PostInviteVerifyRequestBody;

/// Verify body for *POST /invite/verify*
#[derive(Clone, Debug, PartialEq, Eq, HttpRequest)]
#[http_request(
    method = "POST",
    response = opentalk_types_api_v1::rooms::by_room_id::invites::PostInviteVerifyResponseBody,
    path = "/v1/invite/verify"
)]
pub struct PostInviteVerifyRequest(#[http_request(body)] pub PostInviteVerifyRequestBody);
