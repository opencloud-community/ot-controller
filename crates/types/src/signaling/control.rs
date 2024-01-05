// SPDX-FileCopyrightText: OpenTalk GmbH <mail@opentalk.eu>
//
// SPDX-License-Identifier: EUPL-1.2

//! Signaling messages for the `control` namespace

pub mod command;
pub mod event;
pub mod state;

mod associated_participant;
mod participant;
mod waiting_room_state;

pub use associated_participant::AssociatedParticipant;
pub use participant::Participant;
pub use waiting_room_state::{WaitingRoomState, NAMESPACE as WAITING_ROOM_STATE_NAMESPACE};

/// The namespace string for the signaling module
pub const NAMESPACE: &str = "control";
