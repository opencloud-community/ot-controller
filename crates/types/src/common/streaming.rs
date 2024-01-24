// SPDX-FileCopyrightText: OpenTalk GmbH <mail@opentalk.eu>
//
// SPDX-License-Identifier: EUPL-1.2

//! This module contains commonly used types for streaming target endpoints.

use url::Url;

use crate::core::{StreamingKey, StreamingServiceId, StreamingTargetId};

#[allow(unused_imports)]
use crate::imports::*;

/// A streaming service kind
#[derive(Clone, Debug, PartialEq, Eq)]
#[cfg_attr(feature = "clap", derive(clap::ValueEnum))]
#[cfg_attr(
    feature = "serde",
    derive(Serialize, Deserialize),
    serde(tag = "kind", rename_all = "snake_case")
)]
pub enum StreamingServiceKind {
    /// The built-in kind
    Builtin,
    /// A custom kind
    Custom,
    /// A provider kind
    Provider,
}

/// A streaming service
#[derive(Clone, Debug, PartialEq, Eq)]
#[cfg_attr(feature = "serde", derive(Serialize, Deserialize))]
pub struct StreamingService {
    /// The id of the streaming service
    pub id: StreamingServiceId,

    /// The name of the streaming service
    pub name: String,

    /// The kind of the streaming service
    #[cfg_attr(feature = "serde", serde(flatten))]
    pub kind: StreamingServiceKind,

    /// The endpoint url of the streaming service
    #[cfg_attr(
        feature = "serde",
        serde(default, skip_serializing_if = "Option::is_none")
    )]
    pub streaming_url: Option<Url>,

    /// The format of the streaming key
    #[cfg_attr(
        feature = "serde",
        serde(default, skip_serializing_if = "Option::is_none")
    )]
    pub streaming_key_regex: Option<String>,

    /// The format of the url from which the stream can be accessed
    #[cfg_attr(
        feature = "serde",
        serde(default, skip_serializing_if = "Option::is_none")
    )]
    pub public_url_regex: Option<String>,
}

/// A streaming target
#[derive(Clone, Debug, PartialEq, Eq)]
#[cfg_attr(feature = "serde", derive(Serialize, Deserialize))]
pub struct StreamingTarget {
    /// The id of the streaming service this streaming target belongs to
    pub service_id: StreamingServiceId,

    /// The name of the streaming target
    pub name: String,

    /// The endpoint url of the streaming target
    #[cfg_attr(
        feature = "serde",
        serde(default, skip_serializing_if = "Option::is_none")
    )]
    pub streaming_url: Option<Url>,

    /// The streaming key
    #[cfg_attr(
        feature = "serde",
        serde(default, skip_serializing_if = "Option::is_none")
    )]
    pub streaming_key: Option<StreamingKey>,

    /// The url from which the stream can be accessed
    #[cfg_attr(
        feature = "serde",
        serde(default, skip_serializing_if = "Option::is_none")
    )]
    pub public_url: Option<Url>,
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
