// SPDX-FileCopyrightText: OpenTalk GmbH <mail@opentalk.eu>
//
// SPDX-License-Identifier: EUPL-1.2
use std::sync::{Arc, OnceLock};

use async_trait::async_trait;
use opentalk_signaling_core::{SignalingModuleError, SignalingRoomId, VolatileStaticMemoryStorage};
use parking_lot::RwLock;

use super::memory::MemoryWhiteboardState;
use crate::storage::{InitState, SpaceInfo, WhiteboardStorage};

static STATE: OnceLock<Arc<RwLock<MemoryWhiteboardState>>> = OnceLock::new();

fn state() -> &'static Arc<RwLock<MemoryWhiteboardState>> {
    STATE.get_or_init(Default::default)
}

#[async_trait(?Send)]
impl WhiteboardStorage for VolatileStaticMemoryStorage {
    #[tracing::instrument(name = "spacedeck_try_start_init", skip(self))]
    async fn try_start_init(
        &mut self,
        room: SignalingRoomId,
    ) -> Result<Option<InitState>, SignalingModuleError> {
        Ok(state().write().init_get_or_default(room))
    }

    #[tracing::instrument(name = "spacedeck_set_initialized", skip(self, space_info))]
    async fn set_initialized(
        &mut self,
        room: SignalingRoomId,
        space_info: SpaceInfo,
    ) -> Result<(), SignalingModuleError> {
        state().write().set_initialized(room, space_info);
        Ok(())
    }

    #[tracing::instrument(name = "get_spacedeck_init_state", skip(self))]
    async fn get_init_state(
        &mut self,
        room: SignalingRoomId,
    ) -> Result<Option<InitState>, SignalingModuleError> {
        Ok(state().read().get_init_state(room))
    }

    #[tracing::instrument(name = "delete_spacedeck_init_state", skip(self))]
    async fn delete_init_state(
        &mut self,
        room: SignalingRoomId,
    ) -> Result<(), SignalingModuleError> {
        state().write().delete_init_state(room);
        Ok(())
    }
}

#[cfg(test)]
mod tests {
    use opentalk_signaling_core::VolatileStaticMemoryStorage;
    use serial_test::serial;

    use super::{super::super::test_common, state};

    fn storage() -> VolatileStaticMemoryStorage {
        state().write().reset();
        VolatileStaticMemoryStorage
    }

    #[tokio::test]
    #[serial]
    async fn initialization() {
        test_common::initialization(&mut storage()).await;
    }
}
