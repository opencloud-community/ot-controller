// SPDX-FileCopyrightText: OpenTalk GmbH <mail@opentalk.eu>
//
// SPDX-License-Identifier: EUPL-1.2

//! This module contains commonly used types for streaming target endpoints.

use opentalk_types_common::{
    streaming::{StreamingKey, StreamingLink, StreamingTargetId},
    utils::ExampleData,
};
use url::Url;

#[allow(unused_imports)]
use crate::imports::*;

/// A streaming target kind
#[derive(Clone, Debug, PartialEq, Eq)]
#[cfg_attr(
    feature = "serde",
    derive(Serialize, Deserialize),
    serde(tag = "kind", rename_all = "snake_case")
)]
#[cfg_attr(
    feature = "utoipa",
    derive(utoipa::ToSchema),
    schema(example = json!(StreamingTargetKind::example_data()))
)]
pub enum StreamingTargetKind {
    /// The "custom" kind
    Custom {
        /// The endpoint url of the streaming target
        streaming_endpoint: Url,
        /// The streaming key
        streaming_key: StreamingKey,
        /// The url from which the stream can be accessed
        public_url: Url,
    },
}

impl ExampleData for StreamingTargetKind {
    fn example_data() -> Self {
        Self::Custom {
            streaming_endpoint: "https://ingress.streaming.example.com/"
                .parse()
                .expect("url should be valid"),
            streaming_key: StreamingKey::example_data(),
            public_url: "https://streaming.example.com/livestream123"
                .parse()
                .expect("url should be valid"),
        }
    }
}

impl StreamingTargetKind {
    /// Return streaming_endpoint + streaming_key
    pub fn get_stream_target_location(&self) -> Option<Url> {
        match self {
            StreamingTargetKind::Custom {
                streaming_endpoint,
                streaming_key,
                public_url: _,
            } => {
                let mut endpoint = streaming_endpoint.clone();
                if !endpoint.as_str().ends_with('/') {
                    endpoint.set_path(&format!("{}/", endpoint.path()));
                }

                endpoint.join(streaming_key.as_str()).ok()
            }
        }
    }
}

/// A streaming target
#[derive(Clone, Debug, PartialEq, Eq)]
#[cfg_attr(feature = "serde", derive(Serialize, Deserialize))]
#[cfg_attr(
    feature = "utoipa",
    derive(utoipa::ToSchema),
    schema(example = json!(StreamingTarget::example_data()))
)]
pub struct StreamingTarget {
    /// The name of the streaming target
    pub name: String,

    /// The kind of the streaming target
    #[cfg_attr(feature = "serde", serde(flatten))]
    pub kind: StreamingTargetKind,
}

impl ExampleData for StreamingTarget {
    fn example_data() -> Self {
        Self {
            name: "Example Stream".to_string(),
            kind: StreamingTargetKind::example_data(),
        }
    }
}

/// A streaming target which is specific for a Room
#[derive(Clone, Debug, PartialEq, Eq)]
#[cfg_attr(feature = "serde", derive(Serialize, Deserialize))]
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
pub async fn get_public_urls_from_streaming_targets(
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
