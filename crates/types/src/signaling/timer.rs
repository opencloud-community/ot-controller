// SPDX-FileCopyrightText: OpenTalk GmbH <mail@opentalk.eu>
//
// SPDX-License-Identifier: EUPL-1.2

//! Signaling messages for the `timer` namespace

pub mod command;
pub mod event;
pub mod ready_status;
pub mod status;

mod config;
mod kind;
mod timer_id;

pub use config::TimerConfig;
pub use kind::Kind;
pub use timer_id::TimerId;

/// The namespace string for the signaling module
pub const NAMESPACE: &str = "timer";
