// SPDX-FileCopyrightText: OpenTalk GmbH <mail@opentalk.eu>
//
// SPDX-License-Identifier: EUPL-1.2

//! Signaling messages for the `breakout` namespace

mod breakout_room;
mod participant_in_other_room;

pub mod command;
pub mod event;

pub use breakout_room::BreakoutRoom;
pub use participant_in_other_room::ParticipantInOtherRoom;
