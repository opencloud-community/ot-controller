// SPDX-FileCopyrightText: OpenTalk GmbH <mail@opentalk.eu>
//
// SPDX-License-Identifier: EUPL-1.2

//! Signaling messages for the `timer` namespace

pub mod command;
pub mod event;
pub mod ready_status;
pub mod status;

mod config;

pub use config::TimerConfig;
