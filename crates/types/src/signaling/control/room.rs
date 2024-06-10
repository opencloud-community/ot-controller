// SPDX-FileCopyrightText: OpenTalk GmbH <mail@opentalk.eu>
//
// SPDX-License-Identifier: EUPL-1.2

//! Types related to the `JoinSuccess` message in the `control` namespace

use crate::core::RoomId;
#[allow(unused_imports)]
use crate::imports::*;

/// Information about an room
#[derive(Clone, Debug, PartialEq, Eq)]
#[cfg_attr(feature = "serde", derive(Serialize, Deserialize))]
pub struct RoomInfo {
    /// The id of the room
    pub id: RoomId,

    /// The room password
    #[cfg_attr(feature = "serde", serde(skip_serializing_if = "Option::is_none"))]
    pub password: Option<String>,

    /// The room creator
    pub created_by: CreatorInfo,
}

/// Information about the room creator
#[derive(Clone, Debug, PartialEq, Eq)]
#[cfg_attr(feature = "serde", derive(Serialize, Deserialize))]
#[cfg_attr(
    feature = "redis",
    derive(ToRedisArgs, FromRedisValue),
    to_redis_args(serde),
    from_redis_value(serde)
)]
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
