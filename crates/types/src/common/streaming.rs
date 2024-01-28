// SPDX-FileCopyrightText: OpenTalk GmbH <mail@opentalk.eu>
//
// SPDX-License-Identifier: EUPL-1.2

//! This module contains commonly used types for streaming target endpoints.

use url::Url;

use crate::core::{StreamingKey, StreamingTargetId};

#[allow(unused_imports)]
use crate::imports::*;

/// A streaming target kind
#[derive(Clone, Debug, PartialEq, Eq)]
#[cfg_attr(
    feature = "serde",
    derive(Serialize, Deserialize),
    serde(tag = "kind", rename_all = "snake_case")
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

/// A streaming target
#[derive(Clone, Debug, PartialEq, Eq)]
#[cfg_attr(feature = "serde", derive(Serialize, Deserialize))]
pub struct StreamingTarget {
    /// The name of the streaming target
    pub name: String,

    /// The kind of the streaming target
    #[cfg_attr(feature = "serde", serde(flatten))]
    pub kind: StreamingTargetKind,
}

/// A streaming target which is specific for a Room
#[derive(Clone, Debug, PartialEq, Eq)]
#[cfg_attr(feature = "serde", derive(Serialize, Deserialize))]
pub struct RoomStreamingTarget {
    /// The streaming target id
    pub id: StreamingTargetId,

    /// The streaming target
    #[cfg_attr(feature = "serde", serde(flatten))]
    pub streaming_target: StreamingTarget,
}
