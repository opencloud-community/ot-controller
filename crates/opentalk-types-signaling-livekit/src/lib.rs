// SPDX-FileCopyrightText: OpenTalk GmbH <mail@opentalk.eu>
//
// SPDX-License-Identifier: EUPL-1.2

//! Signaling data types for the OpenTalk livekit module.

#![deny(
    bad_style,
    missing_debug_implementations,
    missing_docs,
    overflowing_literals,
    patterns_in_fns_without_body,
    trivial_casts,
    trivial_numeric_casts,
    unsafe_code
)]

pub mod command;
pub mod event;
pub mod state;

// Credentials
mod credentials;
mod microphone_restriction_state;
pub use credentials::Credentials;
pub use microphone_restriction_state::MicrophoneRestrictionState;

/// The namespace string for the signaling module
pub const NAMESPACE: &str = "livekit";

/// Get the id of the signaling module
pub fn module_id() -> opentalk_types_common::modules::ModuleId {
    NAMESPACE.parse().expect("valid module id")
}
