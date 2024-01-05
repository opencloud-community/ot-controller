// SPDX-FileCopyrightText: OpenTalk GmbH <mail@opentalk.eu>
//
// SPDX-License-Identifier: EUPL-1.2

//! Signaling messages for the `chat` namespace

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
