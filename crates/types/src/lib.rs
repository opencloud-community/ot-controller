// SPDX-FileCopyrightText: OpenTalk GmbH <mail@opentalk.eu>
//
// SPDX-License-Identifier: EUPL-1.2

//! Data types for OpenTalk.
//!
//! This crate contains all data types that are used in the OpenTalk
//! web and signaling APIs.
//!
//! # Features
//!
//! ## `default`
//!
//! This is the "easy" way to use this crate, unless you need specific
//! functionalities for the backend, then you should use the `backend`
//! feature instead.
//!
//! Depends on:
//! - `frontend`
//!
//! ## `backend`
//!
//! Set the `backend` feature for using the types anywhere in the backend
//! (e.g., a signaling module, the OpenTalk controller implementation,
//! the OpenTalk room server).
//!
//! Depends on:
//! - `diesel`
//! - `redis`
//! - `kustos`
//! - `serde`
//! - `rand`
//!
//! ## `frontend`
//!
//! Set the `frontend` feature for using the types in a client. Because
//! the `default` feature depends on this, you probably don't need to set it
//! explicitly, unless you have set `default-features = false`.
//!
//! ## `diesel`
//!
//! Adds [Diesel](https://diesel.rs/) type mappings to simple newtypes,
//! so they can be stored in a database through the ORM.
//!
//! Depends on:
//! - `serde`
//!
//! ## `redis`
//!
//! Implements [Redis](https://docs.rs/redis/) `ToRedisArgs` and `FromRedisValue`
//! for types that can be stored on a redis server.
//!
//! Depends on:
//! - `serde`
//!
//! ## `kustos`
//!
//! Annotates identifier newtypes with a kustos resource implementation.
//!
//! ## `rand`
//!
//! Some functions for generating values from random numbers are gated by this flag.
//! These are typically used on the backend for creating new identifiers or tokens.
//!
//! ## `serde`
//!
//! Derives [`serde::Serialize`] and [`serde::Deserialize`] for all types that can be
//! serialized or deserialized for usage in the web and signaling APIs as well as
//! Diesel and Redis.

#![warn(
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

mod macros;
mod utils;

pub mod api;
pub mod common;
pub mod core;
pub mod signaling;
pub mod strings;

mod imports {
    #![allow(unused)]

    #[cfg(feature = "diesel")]
    pub use {
        diesel::{
            deserialize::{FromSql, FromSqlRow},
            expression::AsExpression,
            pg::Pg,
            serialize::ToSql,
        },
        diesel_newtype::DieselNewtype,
    };

    #[cfg(feature = "kustos")]
    pub use kustos_prefix::KustosPrefix;

    #[cfg(feature = "redis")]
    pub use {
        redis::{FromRedisValue, RedisResult, ToRedisArgs},
        redis_args::{FromRedisValue, ToRedisArgs},
    };

    #[cfg(feature = "serde")]
    pub use {
        crate::signaling::{SignalingModuleFrontendData, SignalingModulePeerFrontendData},
        serde::{de, de::DeserializeOwned, Deserialize, Deserializer, Serialize, Serializer},
        validator::{Validate, ValidationError, ValidationErrors},
    };

    #[cfg(feature = "frontend")]
    pub use {client_shared::Request, http::Method};
}
