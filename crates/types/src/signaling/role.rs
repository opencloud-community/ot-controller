// SPDX-FileCopyrightText: OpenTalk GmbH <mail@opentalk.eu>
//
// SPDX-License-Identifier: EUPL-1.2

use strum::{AsRefStr, Display, EnumCount, EnumIter, EnumString, EnumVariantNames, IntoStaticStr};

#[allow(unused_imports)]
use crate::imports::*;

/// Role of the participant inside a room
#[derive(
    Debug,
    Clone,
    Copy,
    PartialEq,
    Eq,
    AsRefStr,
    Display,
    EnumCount,
    EnumIter,
    EnumString,
    EnumVariantNames,
    IntoStaticStr,
)]
#[cfg_attr(
    feature = "redis",
    derive(ToRedisArgs, FromRedisValue),
    to_redis_args(Display),
    from_redis_value(FromStr)
)]
#[cfg_attr(
    feature = "serde",
    derive(Serialize, Deserialize),
    serde(rename_all = "lowercase")
)]
#[strum(serialize_all = "snake_case")]
pub enum Role {
    /// Guest participant without a registered user account
    Guest,

    /// Regular participant with a registered user account
    User,

    /// Participant with a registered user account and moderation permissions
    Moderator,
}

impl Role {
    /// Returns `true` if the role is a [`Role::Moderator`] value.
    pub const fn is_moderator(&self) -> bool {
        matches!(self, Role::Moderator)
    }

    /// Returns `true` if the role is a [`Role::User`] value.
    pub const fn is_user(&self) -> bool {
        matches!(self, Role::User)
    }

    /// Returns `true` if the role is a [`Role::Guest`] value.
    pub const fn is_guest(&self) -> bool {
        matches!(self, Role::Guest)
    }
}
