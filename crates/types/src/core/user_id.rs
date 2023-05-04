// SPDX-FileCopyrightText: OpenTalk GmbH <mail@opentalk.eu>
//
// SPDX-License-Identifier: EUPL-1.2

#[cfg(feature = "kustos")]
use kustos::subject::PolicyUser;

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
    UserId(uuid::Uuid) => diesel::sql_types::Uuid, "/users/"
}

#[cfg(feature = "kustos")]
impl From<UserId> for PolicyUser {
    fn from(id: UserId) -> Self {
        Self::from(id.into_inner())
    }
}
