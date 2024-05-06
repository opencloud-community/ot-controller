// SPDX-FileCopyrightText: OpenTalk GmbH <mail@opentalk.eu>
//
// SPDX-License-Identifier: EUPL-1.2

//! Signaling messages for the `recording_service` namespace

pub mod command;
pub mod event;
pub mod state;

/// The namespace string for this signaling module
pub const NAMESPACE: &str = "recording_service";
