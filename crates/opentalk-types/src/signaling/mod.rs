// SPDX-FileCopyrightText: OpenTalk GmbH <mail@opentalk.eu>
//
// SPDX-License-Identifier: EUPL-1.2

//! This module contains types that are used by the signaling communication
//! (typically through websockets)

pub mod core;
pub mod echo;
pub mod integration;

mod namespaced;

pub use namespaced::NamespacedEvent;
