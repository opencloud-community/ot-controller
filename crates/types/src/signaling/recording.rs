// SPDX-FileCopyrightText: OpenTalk GmbH <mail@opentalk.eu>
//
// SPDX-License-Identifier: EUPL-1.2

//! Signaling messages for the `recording` namespace

mod recording_id;
mod recording_status;

pub mod command;
pub mod event;

pub use recording_id::RecordingId;
pub use recording_status::RecordingStatus;
