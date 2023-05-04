// SPDX-FileCopyrightText: OpenTalk GmbH <mail@opentalk.eu>
//
// SPDX-License-Identifier: EUPL-1.2

#[allow(unused_imports)]
use crate::imports::*;

use super::RecordingId;

/// Status of a recording
#[derive(Debug, Clone, PartialEq, Eq)]
#[cfg_attr(
    feature = "serde",
    derive(Serialize, Deserialize),
    serde(tag = "state", content = "recording_id", rename_all = "snake_case")
)]
#[cfg_attr(
    feature = "redis",
    derive(ToRedisArgs, FromRedisValue),
    to_redis_args(serde),
    from_redis_value(serde)
)]
pub enum RecordingStatus {
    /// Waiting for a recorder to connect and start the recording
    Initializing,
    /// A recorder is connected and capturing the conference
    Recording(RecordingId),
}
