// SPDX-FileCopyrightText: OpenTalk GmbH <mail@opentalk.eu>
//
// SPDX-License-Identifier: EUPL-1.2

//! This module contains types that are used for OpenTalk API V1 streaming target endpoints for specific rooms.

use crate::{
    api::v1::streaming_targets::UpdateStreamingTarget,
    common::streaming::{RoomStreamingTarget, StreamingTarget},
};

#[allow(unused_imports)]
use crate::imports::*;

/// The body of a *GET /rooms/{room_id}/streaming_targets* response
#[derive(Clone, Debug, PartialEq, Eq)]
#[cfg_attr(feature = "serde", derive(Serialize, Deserialize))]
pub struct GetRoomStreamingTargetsResponse(pub Vec<RoomStreamingTarget>);

/// The body of a *POST /rooms/{room_id}/streaming_targets* request
#[derive(Clone, Debug, PartialEq, Eq)]
#[cfg_attr(feature = "serde", derive(Serialize, Deserialize))]
pub struct PostRoomStreamingTargetRequest(pub StreamingTarget);

/// The body of a *POST /rooms/{room_id}/streaming_targets* response
#[derive(Clone, Debug, PartialEq, Eq)]
#[cfg_attr(feature = "serde", derive(Serialize, Deserialize))]
pub struct PostRoomStreamingTargetResponse(pub RoomStreamingTarget);

/// The body of a *GET /rooms/{room_id}/streaming_targets/{streaming_target_id}* response
#[derive(Clone, Debug, PartialEq, Eq)]
#[cfg_attr(feature = "serde", derive(Serialize, Deserialize))]
pub struct GetRoomStreamingTargetResponse(pub RoomStreamingTarget);

/// The body of a *PUT /rooms/{room_id}/streaming_targets/{streaming_target_id}* request
#[derive(Clone, Debug, PartialEq, Eq)]
#[cfg_attr(feature = "serde", derive(Serialize, Deserialize))]
pub struct ChangeRoomStreamingTargetRequest(pub UpdateStreamingTarget);

/// The body of a *PUT /rooms/{room_id}/streaming_targets/{streaming_target_id}* response
#[derive(Clone, Debug, PartialEq, Eq)]
#[cfg_attr(feature = "serde", derive(Serialize, Deserialize))]
pub struct ChangeRoomStreamingTargetResponse(pub RoomStreamingTarget);

#[cfg(test)]
mod test {
    use crate::core::{StreamingKey, StreamingServiceId, StreamingTargetId};

    use super::*;
    use pretty_assertions::assert_eq;
    use serde_json::json;

    #[test]
    fn streaming_target_basic() {
        let expected = json!({
            "id": "00000000-0000-0000-0000-000000000000",
            "service_id": "00000000-0000-0000-0000-000000000000",
            "name": "my streaming target",
            "streaming_url": "http://127.0.0.1/",
            "streaming_key": "1337",
            "public_url": "https://localhost/",
        });

        let produced = serde_json::to_value(RoomStreamingTarget {
            id: StreamingTargetId::nil(),
            streaming_target: StreamingTarget {
                service_id: StreamingServiceId::nil(),
                name: "my streaming target".to_string(),
                streaming_url: Some("http://127.0.0.1/".parse().unwrap()),
                streaming_key: Some(StreamingKey::from("1337".to_string())),
                public_url: Some("https://localhost/".parse().unwrap()),
            },
        })
        .unwrap();

        assert_eq!(expected, produced);
    }
}
