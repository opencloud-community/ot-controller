// SPDX-FileCopyrightText: OpenTalk GmbH <mail@opentalk.eu>
//
// SPDX-License-Identifier: EUPL-1.2

//! Signaling messages for the `control` namespace

pub mod command;
pub mod event;
pub mod room;
pub mod state;

mod reason;
mod waiting_room_state;

use opentalk_types_common::modules::ModuleId;
pub use reason::Reason;
pub use waiting_room_state::{WaitingRoomState, NAMESPACE as WAITING_ROOM_STATE_NAMESPACE};

/// The namespace string for the signaling module
pub const NAMESPACE: &str = "control";

/// Get the id of the signaling module
pub fn module_id() -> ModuleId {
    NAMESPACE.parse().expect("valid module id")
}
