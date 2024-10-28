// SPDX-FileCopyrightText: OpenTalk GmbH <mail@opentalk.eu>
//
// SPDX-License-Identifier: EUPL-1.2

//! This module contains types that are used by the signaling communication
//! (typically through websockets)

pub mod core;
pub mod echo;
pub mod integration;
pub mod media;
pub mod meeting_report;
pub mod shared_folder;
pub mod timer;
pub mod whiteboard;

mod namespaced;

pub use namespaced::{NamespacedCommand, NamespacedEvent};
