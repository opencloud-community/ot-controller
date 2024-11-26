// SPDX-FileCopyrightText: OpenTalk GmbH <mail@opentalk.eu>
//
// SPDX-License-Identifier: EUPL-1.2

use opentalk_types_common::rooms::RoomId;

#[allow(unused_imports)]
use crate::imports::*;

/// POST request to */rooms/{room_id}/start*
#[derive(Clone, Debug, PartialEq, Eq)]
#[cfg_attr(
    feature = "frontend",
    derive(HttpRequest),
    http_request(
        method = "POST",
        response = opentalk_types_api_v1::rooms::by_room_id::RoomsStartResponseBody,
        path = "/rooms/{room_id}/start_invited"
    )
)]
pub struct PostRoomsStartInvitedRequest {
    /// The id of the room for which the request is sent
    pub room_id: RoomId,

    /// The body of the request
    pub body: opentalk_types_api_v1::rooms::by_room_id::PostRoomsStartInvitedRequestBody,
}
