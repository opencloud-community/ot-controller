// SPDX-FileCopyrightText: OpenTalk GmbH <mail@opentalk.eu>
//
// SPDX-License-Identifier: EUPL-1.2

//! Signaling messages for the `recording` namespace

mod recording_id;
mod recording_status;

pub mod command;
pub mod event;
pub mod peer_state;
pub mod state;

pub use recording_id::RecordingId;
pub use recording_status::RecordingStatus;

/// The namespace string for the signaling module
pub const NAMESPACE: &str = "recording";
