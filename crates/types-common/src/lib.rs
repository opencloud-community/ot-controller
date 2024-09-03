// SPDX-FileCopyrightText: OpenTalk GmbH <mail@opentalk.eu>
//
// SPDX-License-Identifier: EUPL-1.2

//! Common data types for OpenTalk.
//!
//! This crate contains data types that are commonly used in the OpenTalk !
//! APIs.

#![deny(
    bad_style,
    missing_debug_implementations,
    missing_docs,
    overflowing_literals,
    patterns_in_fns_without_body,
    trivial_casts,
    trivial_numeric_casts,
    unsafe_code,
    unused,
    unused_extern_crates,
    unused_import_braces,
    unused_qualifications,
    unused_results
)]

pub mod assets;
pub mod auth;
pub mod call_in;
pub mod collections;
pub mod email;
pub mod events;
pub mod features;
pub mod module_resources;
pub mod modules;
pub mod rooms;
pub mod shared_folders;
pub mod streaming;
pub mod tariffs;
pub mod tenants;
pub mod time;
pub mod users;
pub mod utils;

mod macros;

mod imports {
    #![allow(unused)]

    #[cfg(feature = "kustos")]
    pub use opentalk_kustos_prefix::KustosPrefix;
    #[cfg(feature = "serde")]
    pub use serde::{de, de::DeserializeOwned, Deserialize, Deserializer, Serialize, Serializer};
    #[cfg(feature = "diesel")]
    pub use {
        diesel::{
            deserialize::{FromSql, FromSqlRow},
            expression::AsExpression,
            pg::Pg,
            serialize::ToSql,
        },
        opentalk_diesel_newtype::DieselNewtype,
    };
    #[cfg(feature = "redis")]
    pub use {
        redis::{FromRedisValue, RedisResult, ToRedisArgs},
        redis_args::{FromRedisValue, ToRedisArgs},
    };
}
