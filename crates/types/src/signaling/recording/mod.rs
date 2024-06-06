// SPDX-FileCopyrightText: OpenTalk GmbH <mail@opentalk.eu>
//
// SPDX-License-Identifier: EUPL-1.2

//! Signaling messages for the `recording` namespace

mod recording_id;
mod stream_error_reason;
mod stream_kind;
mod stream_status;
mod stream_target;
mod stream_updated;

pub mod command;
pub mod event;
pub mod peer_state;
pub mod state;

pub use recording_id::RecordingId;
pub use stream_error_reason::StreamErrorReason;
pub use stream_kind::{StreamKind, StreamKindSecret};
pub use stream_status::StreamStatus;
pub use stream_target::{StreamTarget, StreamTargetSecret};
pub use stream_updated::StreamUpdated;

/// The namespace string for the signaling module
pub const NAMESPACE: &str = "recording";

/// The feature for allowing recording of meetings
pub const RECORD_FEATURE: &str = "record";

/// The feature for allowing streaming of meetings
pub const STREAM_FEATURE: &str = "stream";
