// SPDX-FileCopyrightText: OpenTalk GmbH <mail@opentalk.eu>
//
// SPDX-License-Identifier: EUPL-1.2

//! Frontend data for `recording` namespace

use opentalk_types_signaling_recording::{StreamKindSecret, StreamStatus, StreamTargetSecret};
use opentalk_types_signaling_recording_service::state::{
    RecordingTarget, StreamStartOption, StreamingTarget,
};

#[allow(unused_imports)]
use crate::imports::*;

/// The target specifier whether a livestream or a recording shall be targeted
#[derive(Debug, Clone, PartialEq, Eq)]
#[cfg_attr(
    feature = "serde",
    derive(Deserialize, Serialize),
    serde(tag = "stream_kind", rename_all = "snake_case")
)]
pub enum RecorderStreamInfo {
    /// Recording target
    Recording(RecordingTarget),
    /// Streaming target
    Streaming(StreamingTarget),
}

impl RecorderStreamInfo {
    /// Returns whether the stream is requested to start.
    pub fn is_start_requested(&self) -> bool {
        match self {
            RecorderStreamInfo::Recording(target) => {
                target.stream_start_options.status == StreamStatus::Starting
            }
            RecorderStreamInfo::Streaming(target) => {
                target.stream_start_options.status == StreamStatus::Starting
            }
        }
    }
}

impl From<StreamTargetSecret> for RecorderStreamInfo {
    fn from(stream_target: StreamTargetSecret) -> RecorderStreamInfo {
        match stream_target.kind {
            StreamKindSecret::Recording => RecorderStreamInfo::Recording(RecordingTarget {
                stream_start_options: StreamStartOption {
                    auto_connect: false,
                    status: stream_target.status.clone(),
                    start_paused: false,
                },
            }),
            StreamKindSecret::Livestream(stream_target_kind) => {
                RecorderStreamInfo::Streaming(StreamingTarget {
                    location: stream_target_kind.get_stream_target_location(),
                    stream_start_options: StreamStartOption {
                        auto_connect: false,
                        status: stream_target.status.clone(),
                        start_paused: false,
                    },
                })
            }
        }
    }
}
