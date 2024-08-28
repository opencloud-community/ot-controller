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
pub mod jobs;
pub mod migrations;
pub mod module_resources;
pub mod rooms;
pub mod sip_configs;
pub mod streaming_targets;
pub mod tariffs;
pub mod tenants;
pub mod users;
pub mod utils;

define_sql_function!(fn lower(x: Text) -> Text);
define_sql_function!(fn levenshtein(x: Text, y: Text) -> Integer);
define_sql_function!(fn soundex(x: Text) -> Text);

// SQL types reexport for schema.rs
pub mod sql_types {

    pub use diesel::sql_types::*;
    pub use opentalk_types::core::{
        StreamingKindType as StreamingKind, TariffStatusType as TariffStatus,
    };
    pub use opentalk_types_common::events::invites::{
        EmailInviteRoleType as EmailInviteRole, EventInviteStatusType as EventInviteStatus,
        InviteRoleType as InviteRole,
    };

    pub use super::{
        events::EventExceptionKindType as EventExceptionKind,
        jobs::{JobStatusType as JobStatus, JobTypeType as JobType, LogLevelType as LogLevel},
    };
}
