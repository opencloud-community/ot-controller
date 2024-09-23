// SPDX-FileCopyrightText: OpenTalk GmbH <mail@opentalk.eu>
//
// SPDX-License-Identifier: EUPL-1.2

//! Frontend data for `recording_service` namespace

use std::collections::BTreeMap;

use opentalk_types_common::streaming::StreamingTargetId;
use opentalk_types_signaling_recording_service::state::RecorderStreamInfo;

#[allow(unused_imports)]
use crate::imports::*;

/// The state of the `recording_service` module.
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
pub struct RecordingServiceState {
    /// The streams to be sent initially to the recorder
    pub streams: BTreeMap<StreamingTargetId, RecorderStreamInfo>,
}

#[cfg(feature = "serde")]
impl SignalingModuleFrontendData for RecordingServiceState {
    const NAMESPACE: Option<&'static str> =
        Some(opentalk_types_signaling_recording_service::NAMESPACE);
}
