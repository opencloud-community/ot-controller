// SPDX-FileCopyrightText: OpenTalk GmbH <mail@opentalk.eu>
//
// SPDX-License-Identifier: EUPL-1.2

//! This module contains types that are used for OpenTalk API V1 streaming target endpoints for specific rooms.

#[allow(unused_imports)]
use crate::imports::*;
use crate::{
    api::v1::streaming_targets::UpdateStreamingTarget,
    common::streaming::{RoomStreamingTarget, StreamingTarget},
    utils::ExampleData,
};

/// The body of a *GET /rooms/{room_id}/streaming_targets* response
#[derive(Clone, Debug, PartialEq, Eq)]
#[cfg_attr(feature = "serde", derive(Serialize, Deserialize))]
#[cfg_attr(
    feature = "utoipa",
    derive(utoipa::ToSchema),
    schema(example = json!(GetRoomStreamingTargetsResponse::example_data()))
)]
pub struct GetRoomStreamingTargetsResponse(pub Vec<RoomStreamingTarget>);

impl ExampleData for GetRoomStreamingTargetsResponse {
    fn example_data() -> Self {
        Self(vec![RoomStreamingTarget::example_data()])
    }
}

/// The body of a *POST /rooms/{room_id}/streaming_targets* request
#[derive(Clone, Debug, PartialEq, Eq)]
#[cfg_attr(feature = "serde", derive(Serialize, Deserialize))]
#[cfg_attr(
    feature = "utoipa",
    derive(utoipa::ToSchema),
    schema(example = json!(PostRoomStreamingTargetRequest::example_data()))
)]
pub struct PostRoomStreamingTargetRequest(pub StreamingTarget);

impl ExampleData for PostRoomStreamingTargetRequest {
    fn example_data() -> Self {
        Self(StreamingTarget::example_data())
    }
}

/// The body of a *POST /rooms/{room_id}/streaming_targets* response
#[derive(Clone, Debug, PartialEq, Eq)]
#[cfg_attr(feature = "serde", derive(Serialize, Deserialize))]
#[cfg_attr(
    feature = "utoipa",
    derive(utoipa::ToSchema),
    schema(example = json!(PostRoomStreamingTargetResponse::example_data()))
)]
pub struct PostRoomStreamingTargetResponse(pub RoomStreamingTarget);

impl ExampleData for PostRoomStreamingTargetResponse {
    fn example_data() -> Self {
        Self(RoomStreamingTarget::example_data())
    }
}

/// The body of a *GET /rooms/{room_id}/streaming_targets/{streaming_target_id}* response
#[derive(Clone, Debug, PartialEq, Eq)]
#[cfg_attr(feature = "serde", derive(Serialize, Deserialize))]
#[cfg_attr(
    feature = "utoipa",
    derive(utoipa::ToSchema),
    schema(example = json!(GetRoomStreamingTargetResponse::example_data()))
)]
pub struct GetRoomStreamingTargetResponse(pub RoomStreamingTarget);

impl ExampleData for GetRoomStreamingTargetResponse {
    fn example_data() -> Self {
        Self(RoomStreamingTarget::example_data())
    }
}

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
    use pretty_assertions::assert_eq;
    use serde_json::json;

    use super::*;
    use crate::{
        common::streaming::StreamingTargetKind,
        core::{StreamingKey, StreamingTargetId},
    };

    #[test]
    fn streaming_target_basic() {
        let expected = json!({
            "id": "00000000-0000-0000-0000-000000000000",
            "name": "my streaming target",
            "kind": "custom",
            "streaming_endpoint": "http://127.0.0.1/",
            "streaming_key": "1337",
            "public_url": "https://localhost/",
        });

        let produced = serde_json::to_value(RoomStreamingTarget {
            id: StreamingTargetId::nil(),
            streaming_target: StreamingTarget {
                name: "my streaming target".to_string(),
                kind: StreamingTargetKind::Custom {
                    streaming_endpoint: "http://127.0.0.1/".parse().unwrap(),
                    streaming_key: StreamingKey::from("1337".to_string()),
                    public_url: "https://localhost/".parse().unwrap(),
                },
            },
        })
        .unwrap();

        assert_eq!(expected, produced);
    }
}
