// SPDX-FileCopyrightText: OpenTalk GmbH <mail@opentalk.eu>
//
// SPDX-License-Identifier: EUPL-1.2

//! This module contains types that are used for OpenTalk API V1 streaming target endpoints.

use opentalk_types_api_v1::rooms::streaming_targets::UpdateStreamingTargetKind;
use opentalk_types_common::{rooms::RoomId, streaming::StreamingTargetId, utils::ExampleData};

#[allow(unused_imports)]
use crate::imports::*;

/// The parameter set for /rooms/{room_id}/streaming_targets/{streaming_target_id}* endpoints
#[derive(Clone, Debug, PartialEq, Eq)]
#[cfg_attr(feature = "serde", derive(Serialize, Deserialize))]
#[cfg_attr(feature = "utoipa", derive(utoipa::IntoParams))]
pub struct RoomAndStreamingTargetId {
    /// The room id for the invite
    pub room_id: RoomId,

    /// The streaming target id
    pub streaming_target_id: StreamingTargetId,
}

/// Data to update a streaming target (only fields with [`Some`] are updated)
#[derive(Clone, Debug, PartialEq, Eq)]
#[cfg_attr(feature = "serde", derive(Serialize, Deserialize))]
#[cfg_attr(
    feature = "utoipa",
    derive(utoipa::ToSchema),
    schema(example = json!(UpdateStreamingTarget::example_data()))
)]
pub struct UpdateStreamingTarget {
    /// The name of the streaming target
    pub name: Option<String>,

    /// The kind of the streaming target
    #[cfg_attr(feature = "serde", serde(flatten))]
    pub kind: Option<UpdateStreamingTargetKind>,
}

impl ExampleData for UpdateStreamingTarget {
    fn example_data() -> Self {
        Self {
            name: Some("My OwnCast Stream".to_string()),
            kind: Some(UpdateStreamingTargetKind::example_data()),
        }
    }
}
