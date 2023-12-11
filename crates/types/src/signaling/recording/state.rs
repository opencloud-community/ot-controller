// SPDX-FileCopyrightText: OpenTalk GmbH <mail@opentalk.eu>
//
// SPDX-License-Identifier: EUPL-1.2

//! Frontend data for `recording` namespace

use std::collections::BTreeMap;

use url::Url;

use super::{StreamKindSecret, StreamStatus, StreamTarget, StreamTargetSecret};

#[allow(unused_imports)]
use crate::{core::StreamingTargetId, imports::*};

/// The state of the `recording` module.
///
/// This struct is sent to the participant in the `join_success` message
/// when they join successfully to the meeting.
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
pub struct RecordingState {
    /// The streaming targets
    pub targets: BTreeMap<StreamingTargetId, StreamTarget>,
}

/// The target specifier whether a livestream or a recording shall be targeted
#[derive(Debug, Clone, PartialEq, Eq)]
#[cfg_attr(
    feature = "serde",
    derive(Deserialize, Serialize),
    serde(tag = "stream_kind", rename_all = "snake_case")
)]
pub enum RecorderStreamInfo {
    /// Recording target
    Recording(RecordingTarget),
    /// Streaming target
    Streaming(StreamingTarget),
}

impl RecorderStreamInfo {
    /// Returns whether the stream is requested to start.
    pub fn is_start_requested(&self) -> bool {
        match self {
            RecorderStreamInfo::Recording(target) => {
                target.stream_start_options.status == StreamStatus::Starting
            }
            RecorderStreamInfo::Streaming(target) => {
                target.stream_start_options.status == StreamStatus::Starting
            }
        }
    }
}

impl From<StreamTargetSecret> for RecorderStreamInfo {
    fn from(stream_target: StreamTargetSecret) -> RecorderStreamInfo {
        match stream_target.kind {
            StreamKindSecret::Recording => RecorderStreamInfo::Recording(RecordingTarget {
                stream_start_options: StreamStartOption {
                    auto_connect: false,
                    status: stream_target.status.clone(),
                    start_paused: false,
                },
            }),
            StreamKindSecret::Livestream(stream_target_kind) => {
                RecorderStreamInfo::Streaming(StreamingTarget {
                    location: stream_target_kind.get_stream_target_location(),
                    stream_start_options: StreamStartOption {
                        auto_connect: false,
                        status: stream_target.status.clone(),
                        start_paused: false,
                    },
                })
            }
        }
    }
}

/// The recorder target
#[derive(Debug, Clone, PartialEq, Eq)]
#[cfg_attr(feature = "serde", derive(Deserialize, Serialize))]
pub struct StreamStartOption {
    /// Whether the stream shall be started automatically
    pub auto_connect: bool,

    /// The status of the stream
    pub status: StreamStatus,

    /// Whether the target stream shall be started as Paused
    pub start_paused: bool,
}

/// The recorder target
#[derive(Debug, Clone, PartialEq, Eq)]
#[cfg_attr(feature = "serde", derive(Deserialize, Serialize))]
pub struct RecordingTarget {
    /// The start options for the target stream
    #[cfg_attr(feature = "serde", serde(flatten))]
    pub stream_start_options: StreamStartOption,
}

/// The streaming target
#[derive(Debug, Clone, PartialEq, Eq)]
#[cfg_attr(feature = "serde", derive(Deserialize, Serialize))]
pub struct StreamingTarget {
    /// The start options for the target stream
    #[cfg_attr(feature = "serde", serde(flatten))]
    pub stream_start_options: StreamStartOption,

    /// The target Url to which the stream shall be streamed to
    #[cfg_attr(
        feature = "serde",
        serde(default, skip_serializing_if = "Option::is_none")
    )]
    pub location: Option<Url>,
}

#[cfg(feature = "serde")]
impl SignalingModuleFrontendData for RecordingState {
    const NAMESPACE: Option<&'static str> = Some(super::NAMESPACE);
}

#[cfg(test)]
mod tests {
    use std::collections::BTreeMap;

    use pretty_assertions::assert_eq;

    use crate::{
        core::StreamingTargetId,
        signaling::recording::{StreamErrorReason, StreamKind, StreamStatus, StreamTarget},
    };

    use super::RecordingState;

    #[test]
    fn recording_state_de_serialize() {
        let json = serde_json::json!({
            "targets": {
                "00000000-0000-0000-0000-000000000000": {
                    "name": "abc123",
                    "streaming_kind": "recording",
                    "status": "active",
                },
                "00000000-0000-0000-0000-000000000001": {
                    "name": "xyz321",
                    "streaming_kind": "livestream",
                    "public_url": "https://localhost/stream_with_me",
                    "status": "error",
                    "reason": {
                        "code": "teapot",
                        "message": "I'm a teapot",
                    },
                }
            },
        });

        let value = RecordingState {
            targets: BTreeMap::from([
                (
                    StreamingTargetId::from_u128(0u128),
                    StreamTarget {
                        name: "abc123".to_owned(),
                        kind: StreamKind::Recording,
                        status: StreamStatus::Active,
                    },
                ),
                (
                    StreamingTargetId::from_u128(1u128),
                    StreamTarget {
                        name: "xyz321".to_owned(),
                        kind: StreamKind::Livestream {
                            public_url: "https://localhost/stream_with_me".parse().unwrap(),
                        },
                        status: StreamStatus::Error {
                            reason: StreamErrorReason {
                                code: "teapot".to_owned(),
                                message: "I'm a teapot".to_owned(),
                            },
                        },
                    },
                ),
            ]),
        };

        let serialized = serde_json::to_value(&value);
        assert!(serialized.is_ok());
        assert_eq!(json, serialized.unwrap(), "Serialized JSON matches");

        let deserialized = serde_json::from_value::<RecordingState>(json);
        assert!(deserialized.is_ok());
        assert_eq!(value, deserialized.unwrap(), "Deserialized JSON matches");
    }
}
