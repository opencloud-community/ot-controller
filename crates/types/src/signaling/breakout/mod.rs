// SPDX-FileCopyrightText: OpenTalk GmbH <mail@opentalk.eu>
//
// SPDX-License-Identifier: EUPL-1.2

//! Signaling messages for the `breakout` namespace

mod associated_participant_in_other_room;
mod breakout_room;
mod participant_in_other_room;

pub mod command;
pub mod event;
pub mod state;

pub use associated_participant_in_other_room::AssociatedParticipantInOtherRoom;
pub use breakout_room::BreakoutRoom;
pub use participant_in_other_room::ParticipantInOtherRoom;

/// The namespace string for the signaling module
pub const NAMESPACE: &str = "breakout";
