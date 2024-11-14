// SPDX-FileCopyrightText: OpenTalk GmbH <mail@opentalk.eu>
//
// SPDX-License-Identifier: EUPL-1.2

//! Signaling events for the `breakout` namespace

mod breakout_event;
mod error;
mod started;

pub use breakout_event::BreakoutEvent;
pub use error::Error;
pub use started::Started;
