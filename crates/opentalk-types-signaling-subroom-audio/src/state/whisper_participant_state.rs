// SPDX-FileCopyrightText: OpenTalk GmbH <mail@opentalk.eu>
//
// SPDX-License-Identifier: EUPL-1.2
use strum::{AsRefStr, Display, EnumCount, EnumIter, EnumString, IntoStaticStr, VariantNames};

/// The invite state for a whisper participant
#[derive(
    Debug,
    Clone,
    Default,
    Copy,
    PartialEq,
    Eq,
    AsRefStr,
    Display,
    EnumCount,
    EnumIter,
    EnumString,
    VariantNames,
    IntoStaticStr,
    PartialOrd,
    Ord,
)]
#[cfg_attr(
    feature = "serde",
    derive(serde::Serialize, serde::Deserialize),
    serde(rename_all = "snake_case")
)]
#[cfg_attr(
    feature = "redis",
    derive(redis_args::ToRedisArgs, redis_args::FromRedisValue),
    to_redis_args(Display),
    from_redis_value(FromStr)
)]
#[strum(serialize_all = "snake_case")]
pub enum WhisperState {
    /// The creator of the whisper group
    Creator,
    /// The participant has been invited but did not reply yet
    #[default]
    Invited,
    /// The participant accepted the invite
    Accepted,
}
