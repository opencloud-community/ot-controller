// SPDX-FileCopyrightText: OpenTalk GmbH <mail@opentalk.eu>
//
// SPDX-License-Identifier: EUPL-1.2

//! This module contains types that are used by the signaling communication
//! (typically through websockets)

pub mod breakout;
pub mod chat;
pub mod common;
pub mod control;
pub mod core;
pub mod echo;
pub mod integration;
pub mod media;
pub mod moderation;
pub mod polls;
pub mod protocol;
pub mod recording;
pub mod shared_folder;
pub mod timer;
pub mod whiteboard;

mod namespaced;
mod role;

pub use namespaced::{NamespacedCommand, NamespacedEvent};
pub use role::Role;
