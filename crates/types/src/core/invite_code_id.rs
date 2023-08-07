// SPDX-FileCopyrightText: OpenTalk GmbH <mail@opentalk.eu>
//
// SPDX-License-Identifier: EUPL-1.2

#[cfg(feature = "kustos")]
use kustos::subject::PolicyInvite;

use uuid::Uuid;

crate::diesel_newtype! {
    feature_gated:

    #[derive(Copy)]
    // If feature `kustos` is enabled, `FromStr` is implemented by the
    // `diesel_newtype!(…)` macro.
    #[cfg_attr(
        not(feature = "kustos"),
        derive(derive_more::FromStr),
    )]
    #[cfg_attr(
        feature = "redis",
        derive(redis_args::ToRedisArgs, redis_args::FromRedisValue),
        to_redis_args(fmt),
        from_redis_value(FromStr)
    )]
    InviteCodeId(uuid::Uuid) => diesel::sql_types::Uuid, "/invites/"
}

impl InviteCodeId {
    /// Create a ZERO invite code id, e.g. for testing purposes
    pub const fn nil() -> Self {
        Self::from(Uuid::nil())
    }

    /// Create a invite code id from a number, e.g. for testing purposes
    pub const fn from_u128(id: u128) -> Self {
        Self::from(Uuid::from_u128(id))
    }

    /// Generate a new random invite code id
    #[cfg(feature = "rand")]
    pub fn generate() -> Self {
        Self::from(Uuid::new_v4())
    }
}

#[cfg(feature = "kustos")]
impl From<InviteCodeId> for PolicyInvite {
    fn from(id: InviteCodeId) -> Self {
        Self::from(id.into_inner())
    }
}
