// SPDX-FileCopyrightText: OpenTalk GmbH <mail@opentalk.eu>
//
// SPDX-License-Identifier: EUPL-1.2

/// A flag to track the participants ready status
#[derive(Default, Debug, PartialEq, Eq, Clone)]
#[cfg_attr(
    feature = "serde",
    derive(serde::Serialize, serde::Deserialize),
    serde(rename_all = "snake_case", tag = "kind")
)]
#[cfg_attr(
    feature = "redis",
    derive(redis_args::ToRedisArgs, redis_args::FromRedisValue)
)]
#[cfg_attr(feature = "redis", to_redis_args(serde), from_redis_value(serde))]
pub struct TimerPeerState {
    /// The ready status of the participant
    pub ready_status: bool,
}

#[cfg(feature = "serde")]
impl opentalk_types_signaling::SignalingModulePeerFrontendData for TimerPeerState {
    const NAMESPACE: Option<&'static str> = Some(crate::NAMESPACE);
}
