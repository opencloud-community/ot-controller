// SPDX-FileCopyrightText: OpenTalk GmbH <mail@opentalk.eu>
//
// SPDX-License-Identifier: EUPL-1.2

//! Signaling data types for the OpenTalk meeting-report module.

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

use opentalk_types_common::modules::ModuleId;

/// The namespace string for the signaling module
pub const NAMESPACE: &str = "meeting_report";

/// Get the id of the signaling module
pub fn module_id() -> ModuleId {
    NAMESPACE.parse().expect("valid module id")
}
