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
pub mod utils;

mod imports {
    #![allow(unused)]

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
}
