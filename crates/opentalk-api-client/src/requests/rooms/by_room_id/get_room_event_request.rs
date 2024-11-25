// SPDX-FileCopyrightText: OpenTalk GmbH <mail@opentalk.eu>
//
// SPDX-License-Identifier: EUPL-1.2

use http_request_derive::HttpRequest;
use opentalk_types_api_v1::rooms::by_room_id::GetRoomEventResponseBody;
use opentalk_types_common::rooms::RoomId;

/// A GET request to the `/rooms/<room_id>/event` endpoint
#[derive(Clone, Debug, PartialEq, Eq, HttpRequest)]
#[    http_request(
        method = "GET",
        response = GetRoomEventResponseBody,
        path = "/v1/rooms/{0}/event"
)]
pub struct GetRoomEventRequest(pub RoomId);
