// SPDX-FileCopyrightText: OpenTalk GmbH <mail@opentalk.eu>
//
// SPDX-License-Identifier: EUPL-1.2

use std::collections::{BTreeMap, BTreeSet};

use async_trait::async_trait;
use opentalk_signaling_core::{SignalingModuleError, SignalingRoomId};
use opentalk_types::{
    core::StreamingTargetId,
    signaling::recording::{StreamStatus, StreamTargetSecret},
};

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

    async fn set_stream(
        &mut self,
        room: SignalingRoomId,
        target: StreamingTargetId,
        stream_target: StreamTargetSecret,
    ) -> Result<(), SignalingModuleError>;

    async fn get_streams(
        &mut self,
        room: SignalingRoomId,
    ) -> Result<BTreeMap<StreamingTargetId, StreamTargetSecret>, SignalingModuleError>;

    async fn get_stream(
        &mut self,
        room: SignalingRoomId,
        target: StreamingTargetId,
    ) -> Result<StreamTargetSecret, SignalingModuleError>;

    async fn stream_exists(
        &mut self,
        room: SignalingRoomId,
        target: StreamingTargetId,
    ) -> Result<bool, SignalingModuleError>;

    #[tracing::instrument(level = "debug", skip(self))]
    async fn streams_contain_status(
        &mut self,
        room: SignalingRoomId,
        stati: BTreeSet<StreamStatus>,
    ) -> Result<bool, SignalingModuleError> {
        let found_states = self
            .get_streams(room)
            .await?
            .values()
            .map(|target| target.status.clone())
            .collect();
        Ok(stati.intersection(&found_states).next().is_some())
    }
}
