// SPDX-FileCopyrightText: OpenTalk GmbH <mail@opentalk.eu>
//
// SPDX-License-Identifier: EUPL-1.2

//! Frontend data for `recording_service` namespace

use std::collections::BTreeMap;

use opentalk_types_common::streaming::StreamingTargetId;

use super::RecorderStreamInfo;

/// The state of the `recording_service` module.
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
pub struct RecordingServiceState {
    /// The streams to be sent initially to the recorder
    pub streams: BTreeMap<StreamingTargetId, RecorderStreamInfo>,
}

#[cfg(feature = "serde")]
impl opentalk_types_signaling::SignalingModuleFrontendData for RecordingServiceState {
    const NAMESPACE: Option<&'static str> = Some(crate::NAMESPACE);
}
