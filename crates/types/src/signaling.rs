// SPDX-FileCopyrightText: OpenTalk GmbH <mail@opentalk.eu>
//
// SPDX-License-Identifier: EUPL-1.2

//! This module contains types that are used by the signaling communication
//! (typically through websockets)

mod namespaced;
mod role;

pub use namespaced::{NamespacedCommand, NamespacedEvent};
pub use role::Role;
