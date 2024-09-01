// SPDX-FileCopyrightText: OpenTalk GmbH <mail@opentalk.eu>
//
// SPDX-License-Identifier: EUPL-1.2

//! Signaling messages for the `moderation` namespace

mod kick_scope;

pub mod command;
pub mod event;
pub mod state;

pub use kick_scope::KickScope;
use opentalk_types_common::modules::ModuleId;

/// The namespace string for the signaling module
pub const NAMESPACE: &str = "moderation";

/// Get the id of the signaling module
pub fn module_id() -> ModuleId {
    NAMESPACE.parse().expect("valid module id")
}
