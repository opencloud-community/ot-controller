// SPDX-FileCopyrightText: OpenTalk GmbH <mail@opentalk.eu>
//
// SPDX-License-Identifier: EUPL-1.2

use opentalk_types_common::rooms::RoomId;

use super::PostRoomsStartRequestBody;
#[allow(unused_imports)]
use crate::imports::*;

/// POST request to */rooms/{room_id}/start*
#[derive(Clone, Debug, PartialEq, Eq)]
#[cfg_attr(
    feature = "frontend",
    derive(HttpRequest),
    http_request(
        method = "POST",
        response = super::RoomsStartResponse,
        path = "/rooms/{room_id}/start"
    )
)]
pub struct PostRoomsStartRequest {
    /// The id of the room for which the request is sent
    pub room_id: RoomId,

    /// The body of the request
    pub body: PostRoomsStartRequestBody,
}
