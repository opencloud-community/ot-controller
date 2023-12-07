// SPDX-FileCopyrightText: OpenTalk GmbH <mail@opentalk.eu>
//
// SPDX-License-Identifier: EUPL-1.2

crate::diesel_newtype! {
    feature_gated:

    #[cfg_attr(
        feature="redis",
        derive(redis_args::ToRedisArgs, redis_args::FromRedisValue),
        to_redis_args(fmt = "{0}"),
        from_redis_value(FromStr)
    )]
    #[derive(derive_more::From, derive_more::Into, derive_more::FromStr)]
    StreamingKey(String) => diesel::sql_types::Text
}
