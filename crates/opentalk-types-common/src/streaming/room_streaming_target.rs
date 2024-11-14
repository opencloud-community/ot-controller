// SPDX-FileCopyrightText: OpenTalk GmbH <mail@opentalk.eu>
//
// SPDX-License-Identifier: EUPL-1.2

//! This module contains commonly used types for streaming target endpoints.

use crate::{
    streaming::{StreamingLink, StreamingTarget, StreamingTargetId, StreamingTargetKind},
    utils::ExampleData,
};

/// A streaming target which is specific for a Room
#[derive(Clone, Debug, PartialEq, Eq)]
#[cfg_attr(feature = "serde", derive(serde::Serialize, serde::Deserialize))]
#[cfg_attr(
    feature = "utoipa",
    derive(utoipa::ToSchema),
    schema(example = json!(RoomStreamingTarget::example_data()))
)]
pub struct RoomStreamingTarget {
    /// The streaming target id
    pub id: StreamingTargetId,

    /// The streaming target
    #[cfg_attr(feature = "serde", serde(flatten))]
    pub streaming_target: StreamingTarget,
}

impl ExampleData for RoomStreamingTarget {
    fn example_data() -> Self {
        Self {
            id: StreamingTargetId::example_data(),
            streaming_target: StreamingTarget::example_data(),
        }
    }
}

/// Extracts the public URLs from streaming targets
pub async fn get_public_urls_from_room_streaming_targets(
    streaming_targets: Vec<RoomStreamingTarget>,
) -> Vec<StreamingLink> {
    streaming_targets
        .into_iter()
        .map(|target| match target.streaming_target.kind {
            StreamingTargetKind::Custom {
                streaming_endpoint: _,
                streaming_key: _,
                public_url,
            } => StreamingLink {
                name: target.streaming_target.name,
                url: public_url,
            },
        })
        .collect()
}
