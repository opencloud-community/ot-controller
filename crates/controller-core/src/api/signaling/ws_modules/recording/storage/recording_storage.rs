// SPDX-FileCopyrightText: OpenTalk GmbH <mail@opentalk.eu>
//
// SPDX-License-Identifier: EUPL-1.2

use std::collections::BTreeMap;

use async_trait::async_trait;
use opentalk_signaling_core::{SignalingModuleError, SignalingRoomId};
use opentalk_types::{core::StreamingTargetId, signaling::recording::StreamTargetSecret};

#[async_trait(?Send)]
pub(crate) trait RecordingStorage {
    async fn is_streaming_initialized(
        &mut self,
        room: SignalingRoomId,
    ) -> Result<bool, SignalingModuleError>;

    async fn set_streams(
        &mut self,
        room: SignalingRoomId,
        target_streams: &BTreeMap<StreamingTargetId, StreamTargetSecret>,
    ) -> Result<(), SignalingModuleError>;
}
