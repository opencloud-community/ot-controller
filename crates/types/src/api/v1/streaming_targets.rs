// SPDX-FileCopyrightText: OpenTalk GmbH <mail@opentalk.eu>
//
// SPDX-License-Identifier: EUPL-1.2

//! This module contains types that are used for OpenTalk API V1 streaming target endpoints.

use crate::core::{RoomId, StreamingKey, StreamingTargetId};
use url::Url;

#[allow(unused_imports)]
use crate::imports::*;

/// The parameter set for /rooms/{room_id}/streaming_targets/{streaming_target_id}* endpoints
#[derive(Clone, Debug, PartialEq, Eq)]
#[cfg_attr(feature = "serde", derive(Serialize, Deserialize))]
pub struct RoomAndStreamingTargetId {
    /// The room id for the invite
    pub room_id: RoomId,

    /// The streaming target id
    pub streaming_target_id: StreamingTargetId,
}

/// Data to update a streaming target (only fields with [`Some`] are updated)
#[derive(Clone, Debug, PartialEq, Eq)]
#[cfg_attr(feature = "serde", derive(Serialize, Deserialize))]
pub struct UpdateStreamingTarget {
    /// The name of the streaming target
    pub name: Option<String>,

    /// The kind of the streaming target
    #[cfg_attr(feature = "serde", serde(flatten))]
    pub kind: Option<UpdateStreamingTargetKind>,
}

/// Data to update a streaming target kind (only fields with [`Some`] are updated)
#[derive(Clone, Debug, PartialEq, Eq)]
#[cfg_attr(
    feature = "serde",
    derive(Serialize, Deserialize),
    serde(tag = "kind", rename_all = "snake_case")
)]
pub enum UpdateStreamingTargetKind {
    /// The "custom" kind
    Custom {
        /// The endpoint url of the streaming target
        #[cfg_attr(feature = "serde", serde(skip_serializing_if = "Option::is_none"))]
        streaming_endpoint: Option<Url>,
        /// The streaming key
        #[cfg_attr(feature = "serde", serde(skip_serializing_if = "Option::is_none"))]
        streaming_key: Option<StreamingKey>,
        /// The url from which the stream can be accessed
        #[cfg_attr(feature = "serde", serde(skip_serializing_if = "Option::is_none"))]
        public_url: Option<Url>,
    },
}
