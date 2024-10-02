// SPDX-FileCopyrightText: OpenTalk GmbH <mail@opentalk.eu>
//
// SPDX-License-Identifier: EUPL-1.2

use std::{
    collections::BTreeMap,
    sync::{Arc, OnceLock},
};

use async_trait::async_trait;
use opentalk_signaling_core::{
    NotFoundSnafu, SignalingModuleError, SignalingRoomId, VolatileStaticMemoryStorage,
};
use opentalk_types::signaling::recording::StreamTargetSecret;
use opentalk_types_common::streaming::StreamingTargetId;
use parking_lot::RwLock;
use snafu::OptionExt as _;

use super::memory::MemoryRecordingState;
use crate::api::signaling::recording::storage::RecordingStorage;

static STATE: OnceLock<Arc<RwLock<MemoryRecordingState>>> = OnceLock::new();

fn state() -> &'static Arc<RwLock<MemoryRecordingState>> {
    STATE.get_or_init(Default::default)
}

#[async_trait(?Send)]
impl RecordingStorage for VolatileStaticMemoryStorage {
    #[tracing::instrument(level = "debug", skip(self))]
    async fn is_streaming_initialized(
        &mut self,
        room: SignalingRoomId,
    ) -> Result<bool, SignalingModuleError> {
        Ok(state().read().is_streaming_initialized(room))
    }

    #[tracing::instrument(level = "debug", skip(self))]
    async fn set_streams(
        &mut self,
        room: SignalingRoomId,
        target_streams: &BTreeMap<StreamingTargetId, StreamTargetSecret>,
    ) -> Result<(), SignalingModuleError> {
        state().write().set_streams(room, target_streams);
        Ok(())
    }

    #[tracing::instrument(level = "debug", skip(self))]
    async fn set_stream(
        &mut self,
        room: SignalingRoomId,
        target: StreamingTargetId,
        stream_target: StreamTargetSecret,
    ) -> Result<(), SignalingModuleError> {
        state().write().set_stream(room, target, stream_target);
        Ok(())
    }

    #[tracing::instrument(level = "debug", skip(self))]
    async fn get_streams(
        &mut self,
        room: SignalingRoomId,
    ) -> Result<BTreeMap<StreamingTargetId, StreamTargetSecret>, SignalingModuleError> {
        Ok(state().read().get_streams(room))
    }

    #[tracing::instrument(level = "debug", skip(self))]
    async fn get_stream(
        &mut self,
        room: SignalingRoomId,
        target: StreamingTargetId,
    ) -> Result<StreamTargetSecret, SignalingModuleError> {
        state()
            .read()
            .get_stream(room, target)
            .with_context(|| NotFoundSnafu {
                message: format!("could not find stream {target} in room {room}"),
            })
    }

    #[tracing::instrument(level = "debug", skip(self))]
    async fn stream_exists(
        &mut self,
        room: SignalingRoomId,
        target: StreamingTargetId,
    ) -> Result<bool, SignalingModuleError> {
        Ok(state().read().stream_exists(room, target))
    }

    #[tracing::instrument(level = "debug", skip(self))]
    async fn delete_all_streams(
        &mut self,
        room: SignalingRoomId,
    ) -> Result<(), SignalingModuleError> {
        state().write().delete_all_streams(room);
        Ok(())
    }
}

#[cfg(test)]
mod tests {
    use opentalk_signaling_core::VolatileStaticMemoryStorage;
    use serial_test::serial;

    use super::{super::super::test_common, state};

    async fn storage() -> VolatileStaticMemoryStorage {
        state().write().reset();
        VolatileStaticMemoryStorage
    }

    #[tokio::test]
    #[serial]
    async fn streams() {
        test_common::streams(&mut storage().await).await;
    }

    #[tokio::test]
    #[serial]
    async fn streams_contain_status() {
        test_common::streams_contain_status(&mut storage().await).await;
    }

    #[tokio::test]
    #[serial]
    async fn update_streams_status() {
        test_common::update_streams_status(&mut storage().await).await;
    }
}
