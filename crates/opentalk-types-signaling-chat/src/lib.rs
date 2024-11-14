// SPDX-FileCopyrightText: OpenTalk GmbH <mail@opentalk.eu>
//
// SPDX-License-Identifier: EUPL-1.2

//! Signaling data types for the OpenTalk chat module.

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

pub mod command;
pub mod event;
pub mod peer_state;
pub mod state;

mod message_id;
mod scope;

pub use message_id::MessageId;
pub use scope::Scope;

/// The namespace string for the signaling module
pub const NAMESPACE: &str = "chat";

/// Get the id of the signaling module
pub fn module_id() -> opentalk_types_common::modules::ModuleId {
    NAMESPACE.parse().expect("valid module id")
}
