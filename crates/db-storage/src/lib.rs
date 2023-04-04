// SPDX-FileCopyrightText: OpenTalk GmbH <mail@opentalk.eu>
//
// SPDX-License-Identifier: EUPL-1.2

#![allow(clippy::extra_unused_lifetimes)]

//! Contains the database ORM and database migrations for the controller/storage
//! Builds upon opentalk-database

#[macro_use]
extern crate diesel;

// postgres functions
use diesel::sql_types::Text;

#[macro_use]
mod macros;
mod schema;

pub mod assets;
pub mod events;
pub mod groups;
pub mod invites;
pub mod migrations;
pub mod rooms;
pub mod sip_configs;
pub mod tariffs;
pub mod tenants;
pub mod users;
pub mod utils;

sql_function!(fn lower(x: Text) -> Text);
sql_function!(fn levenshtein(x: Text, y: Text) -> Integer);
sql_function!(fn soundex(x: Text) -> Text);

// SQL types reexport for schema.rs
pub mod sql_types {
    pub use super::events::EventExceptionKindType as Event_exception_kind;
    pub use super::events::EventInviteStatusType as Event_invite_status;
    pub use diesel::sql_types::*;
}
