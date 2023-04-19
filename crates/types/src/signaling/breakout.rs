// SPDX-FileCopyrightText: OpenTalk GmbH <mail@opentalk.eu>
//
// SPDX-License-Identifier: EUPL-1.2

//! Signaling messages for the `breakout` namespace

mod breakout_room;

pub mod command;
pub mod event;

pub use breakout_room::BreakoutRoom;
