// SPDX-FileCopyrightText: OpenTalk GmbH <mail@opentalk.eu>
//
// SPDX-License-Identifier: EUPL-1.2

//! Signaling data types for the OpenTalk recording module.

#![deny(
    bad_style,
    missing_debug_implementations,
    missing_docs,
    overflowing_literals,
    patterns_in_fns_without_body,
    trivial_casts,
    trivial_numeric_casts,
    unsafe_code,
    unused,
    unused_extern_crates,
    unused_import_braces,
    unused_qualifications,
    unused_results
)]

pub mod command;
pub mod event;
pub mod peer_state;
pub mod state;

mod recording_id;
mod stream_error_reason;
mod stream_kind;
mod stream_kind_secret;
mod stream_status;
mod stream_target;
mod stream_target_secret;
mod stream_updated;

use opentalk_types_common::{features::FeatureId, modules::ModuleId};
pub use recording_id::RecordingId;
pub use stream_error_reason::StreamErrorReason;
pub use stream_kind::StreamKind;
pub use stream_kind_secret::StreamKindSecret;
pub use stream_status::StreamStatus;
pub use stream_target::StreamTarget;
pub use stream_target_secret::StreamTargetSecret;
pub use stream_updated::StreamUpdated;

/// The namespace string for the signaling module
pub const NAMESPACE: &str = "recording";

/// The feature for allowing recording of meetings
pub const RECORD_FEATURE: &str = "record";

/// The feature for allowing streaming of meetings
pub const STREAM_FEATURE: &str = "stream";

/// Get the id of the signaling module
pub fn module_id() -> ModuleId {
    NAMESPACE.parse().expect("valid module id")
}

/// Get the id of the record feature
pub fn record_feature() -> FeatureId {
    RECORD_FEATURE.parse().expect("valid feature id")
}

/// Get the id of the stream feature
pub fn stream_feature() -> FeatureId {
    STREAM_FEATURE.parse().expect("valid feature id")
}
