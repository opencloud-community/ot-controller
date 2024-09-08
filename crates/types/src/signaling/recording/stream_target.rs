// SPDX-FileCopyrightText: OpenTalk GmbH <mail@opentalk.eu>
//
// SPDX-License-Identifier: EUPL-1.2

use opentalk_types_signaling_recording::{StreamKindSecret, StreamStatus};

use super::StreamKind;
#[allow(unused_imports)]
use crate::imports::*;

/// The state information about a stream target
#[derive(Debug, Clone, PartialEq, Eq)]
#[cfg_attr(
    feature = "serde",
    derive(Serialize, Deserialize),
    serde(rename_all = "snake_case")
)]
#[cfg_attr(
    feature = "redis",
    derive(ToRedisArgs, FromRedisValue),
    to_redis_args(serde),
    from_redis_value(serde)
)]
pub struct StreamTarget {
    /// The name of the stream
    pub name: String,
    /// The kind of the stream
    #[cfg_attr(feature = "serde", serde(flatten))]
    pub kind: StreamKind,
    /// The state of the stream
    #[cfg_attr(feature = "serde", serde(flatten))]
    pub status: StreamStatus,
}

/// The state information about a stream target
#[derive(Debug, Clone, PartialEq, Eq)]
#[cfg_attr(
    feature = "serde",
    derive(Serialize, Deserialize),
    serde(rename_all = "snake_case")
)]
#[cfg_attr(
    feature = "redis",
    derive(ToRedisArgs, FromRedisValue),
    to_redis_args(serde),
    from_redis_value(serde)
)]
pub struct StreamTargetSecret {
    /// The name of the stream
    pub name: String,
    /// The kind of the stream including the secrets
    #[cfg_attr(feature = "serde", serde(flatten))]
    pub kind: StreamKindSecret,
    /// The current state of the stream
    #[cfg_attr(feature = "serde", serde(flatten))]
    pub status: StreamStatus,
}

impl StreamTargetSecret {
    /// Creates a recording stream target
    pub fn recording() -> Self {
        Self {
            name: "Recording".to_string(),
            kind: StreamKindSecret::Recording,
            status: StreamStatus::Inactive,
        }
    }
}

impl From<StreamTargetSecret> for StreamTarget {
    fn from(val: StreamTargetSecret) -> Self {
        Self {
            name: val.name.to_owned(),
            kind: val.kind.into(),
            status: val.status,
        }
    }
}
