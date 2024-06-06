// SPDX-FileCopyrightText: OpenTalk GmbH <mail@opentalk.eu>
//
// SPDX-License-Identifier: EUPL-1.2

//! Signaling messages for the `moderation` namespace

mod kick_scope;

pub mod command;
pub mod event;
pub mod state;

pub use kick_scope::KickScope;

/// The namespace string for the signaling module
pub const NAMESPACE: &str = "moderation";
