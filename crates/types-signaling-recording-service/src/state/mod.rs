// SPDX-FileCopyrightText: OpenTalk GmbH <mail@opentalk.eu>
//
// SPDX-License-Identifier: EUPL-1.2

//! Frontend data for `recording_service` namespace

mod recording_target;
mod stream_start_option;
mod streaming_target;

pub use recording_target::RecordingTarget;
pub use stream_start_option::StreamStartOption;
pub use streaming_target::StreamingTarget;
