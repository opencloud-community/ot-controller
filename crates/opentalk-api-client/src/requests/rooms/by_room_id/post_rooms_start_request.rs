// SPDX-FileCopyrightText: OpenTalk GmbH <mail@opentalk.eu>
//
// SPDX-License-Identifier: EUPL-1.2

use http_request_derive::HttpRequest;
use opentalk_types_api_v1::rooms::by_room_id::{PostRoomsStartRequestBody, RoomsStartResponseBody};
use opentalk_types_common::rooms::RoomId;

/// POST request to */rooms/{room_id}/start*
#[derive(Clone, Debug, PartialEq, Eq, HttpRequest)]
#[
    http_request(
        method = "POST",
        response = RoomsStartResponseBody,
        path = "/rooms/{room_id}/start"
)]
pub struct PostRoomsStartRequest {
    /// The id of the room for which the request is sent
    pub room_id: RoomId,

    /// The body of the request
    pub body: PostRoomsStartRequestBody,
}
