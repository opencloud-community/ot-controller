// SPDX-FileCopyrightText: OpenTalk GmbH <mail@opentalk.eu>
//
// SPDX-License-Identifier: EUPL-1.2

//! Database implementation of the data storage facade.

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

mod asset;
mod database_connection;
mod database_connection_pool;
mod event;
mod event_invite;
mod event_shared_folder;
mod event_training_participation_report;
mod group;
mod job_execution;
mod module_resource;
mod room;
mod room_invite;
mod room_sip_config;
mod room_streaming_target;
mod tariff;
mod tenant;
mod transaction_manager;
mod user;

pub use database_connection::DatabaseConnection;
pub use database_connection_pool::DatabaseConnectionPool;
use opentalk_inventory::Result;
