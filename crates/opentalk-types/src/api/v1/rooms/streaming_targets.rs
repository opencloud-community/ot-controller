// SPDX-FileCopyrightText: OpenTalk GmbH <mail@opentalk.eu>
//
// SPDX-License-Identifier: EUPL-1.2

//! This module contains types that are used for OpenTalk API V1 streaming target endpoints for specific rooms.

use opentalk_types_common::{
    streaming::{RoomStreamingTarget, RoomStreamingTargetResource},
    utils::ExampleData,
};

use crate::api::v1::streaming_targets::UpdateStreamingTarget;
#[allow(unused_imports)]
use crate::imports::*;

/// The body of a *GET /rooms/{room_id}/streaming_targets/{streaming_target_id}* response
#[derive(Clone, Debug, PartialEq, Eq)]
#[cfg_attr(feature = "serde", derive(Serialize, Deserialize))]
#[cfg_attr(
    feature = "utoipa",
    derive(utoipa::ToSchema),
    schema(example = json!(GetRoomStreamingTargetResponse::example_data()))
)]
pub struct GetRoomStreamingTargetResponse(pub RoomStreamingTargetResource);

impl ExampleData for GetRoomStreamingTargetResponse {
    fn example_data() -> Self {
        Self(RoomStreamingTargetResource::example_data())
    }
}

/// The body of a *PUT /rooms/{room_id}/streaming_targets/{streaming_target_id}* request
#[derive(Clone, Debug, PartialEq, Eq)]
#[cfg_attr(feature = "serde", derive(Serialize, Deserialize))]
#[cfg_attr(
    feature = "utoipa",
    derive(utoipa::ToSchema),
    schema(example = json!(ChangeRoomStreamingTargetRequest::example_data()))
)]
pub struct ChangeRoomStreamingTargetRequest(pub UpdateStreamingTarget);

impl ExampleData for ChangeRoomStreamingTargetRequest {
    fn example_data() -> Self {
        Self(UpdateStreamingTarget::example_data())
    }
}

/// The body of a *PUT /rooms/{room_id}/streaming_targets/{streaming_target_id}* response
#[derive(Clone, Debug, PartialEq, Eq)]
#[cfg_attr(feature = "serde", derive(Serialize, Deserialize))]
#[cfg_attr(
    feature = "utoipa",
    derive(utoipa::ToSchema),
    schema(example = json!(ChangeRoomStreamingTargetResponse::example_data()))
)]
pub struct ChangeRoomStreamingTargetResponse(pub RoomStreamingTarget);

impl ExampleData for ChangeRoomStreamingTargetResponse {
    fn example_data() -> Self {
        Self(RoomStreamingTarget::example_data())
    }
}
