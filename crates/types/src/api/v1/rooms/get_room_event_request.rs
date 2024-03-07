// SPDX-FileCopyrightText: OpenTalk GmbH <mail@opentalk.eu>
//
// SPDX-License-Identifier: EUPL-1.2

use crate::core::RoomId;

#[allow(unused_imports)]
use crate::imports::*;

/// A GET request to the `/rooms/<room_id>/event` endpoint
#[derive(Clone, Debug, PartialEq, Eq)]
#[cfg_attr(
    feature = "frontend",
    derive(HttpRequest),
    http_request(
        method = "GET",
        response = super::GetRoomEventResponse,
        path = "/v1/rooms/{0}/event"
    )
)]
pub struct GetRoomEventRequest(pub RoomId);
