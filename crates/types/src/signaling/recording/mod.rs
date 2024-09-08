// SPDX-FileCopyrightText: OpenTalk GmbH <mail@opentalk.eu>
//
// SPDX-License-Identifier: EUPL-1.2

//! Signaling messages for the `recording` namespace

mod stream_target;
mod stream_updated;

pub mod command;
pub mod event;
pub mod peer_state;
pub mod state;

pub use stream_target::StreamTarget;
pub use stream_updated::StreamUpdated;
