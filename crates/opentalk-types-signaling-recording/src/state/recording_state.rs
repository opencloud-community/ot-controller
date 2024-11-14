// SPDX-FileCopyrightText: OpenTalk GmbH <mail@opentalk.eu>
//
// SPDX-License-Identifier: EUPL-1.2

use std::collections::BTreeMap;

use opentalk_types_common::streaming::StreamingTargetId;

use crate::StreamTarget;

/// The state of the `recording` module.
///
/// This struct is sent to the participant in the `join_success` message
/// when they join successfully to the meeting.
#[derive(Debug, Clone, PartialEq, Eq)]
#[cfg_attr(
    feature = "serde",
    derive(serde::Serialize, serde::Deserialize),
    serde(rename_all = "snake_case")
)]
#[cfg_attr(
    feature = "redis",
    derive(redis_args::ToRedisArgs, redis_args::FromRedisValue)
)]
#[cfg_attr(feature = "redis", to_redis_args(serde), from_redis_value(serde))]
pub struct RecordingState {
    /// The streaming targets
    pub targets: BTreeMap<StreamingTargetId, StreamTarget>,
}

#[cfg(feature = "serde")]
impl opentalk_types_signaling::SignalingModuleFrontendData for RecordingState {
    const NAMESPACE: Option<&'static str> = Some(crate::NAMESPACE);
}

#[cfg(all(test, feature = "serde"))]
mod serde_tests {
    use std::collections::BTreeMap;

    use opentalk_types_common::streaming::StreamingTargetId;
    use pretty_assertions::assert_eq;

    use super::RecordingState;
    use crate::{StreamErrorReason, StreamKind, StreamStatus, StreamTarget};

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
