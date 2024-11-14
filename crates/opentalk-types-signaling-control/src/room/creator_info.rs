// SPDX-FileCopyrightText: OpenTalk GmbH <mail@opentalk.eu>
//
// SPDX-License-Identifier: EUPL-1.2

/// Information about the room creator
#[derive(Clone, Debug, PartialEq, Eq)]
#[cfg_attr(feature = "serde", derive(serde::Serialize, serde::Deserialize))]
#[cfg_attr(
    feature = "redis",
    derive(redis_args::ToRedisArgs, redis_args::FromRedisValue)
)]
#[cfg_attr(feature = "redis", to_redis_args(serde), from_redis_value(serde))]
pub struct CreatorInfo {
    /// Optional title of the creator
    pub title: String,

    /// The creators first name
    pub firstname: String,

    /// The creators last name
    pub lastname: String,

    /// The creators display name
    pub display_name: String,

    /// The creators avatar url
    pub avatar_url: String,
}
