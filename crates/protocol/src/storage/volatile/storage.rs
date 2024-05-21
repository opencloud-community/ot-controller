// SPDX-FileCopyrightText: OpenTalk GmbH <mail@opentalk.eu>
//
// SPDX-License-Identifier: EUPL-1.2

use std::sync::{Arc, OnceLock};

use async_trait::async_trait;
use opentalk_signaling_core::{SignalingModuleError, SignalingRoomId, VolatileStaticMemoryStorage};
use parking_lot::RwLock;

use super::memory::MemoryProtocolState;
use crate::storage::{protocol_storage::ProtocolStorage, redis::InitState};

static STATE: OnceLock<Arc<RwLock<MemoryProtocolState>>> = OnceLock::new();

fn state() -> &'static Arc<RwLock<MemoryProtocolState>> {
    STATE.get_or_init(Default::default)
}

#[async_trait(?Send)]
impl ProtocolStorage for VolatileStaticMemoryStorage {
    #[tracing::instrument(name = "set_protocol_group", skip(self))]
    async fn group_set(
        &mut self,
        room: SignalingRoomId,
        group: &str,
    ) -> Result<(), SignalingModuleError> {
        state().write().group_set(room, group);
        Ok(())
    }

    #[tracing::instrument(name = "get_protocol_group", skip(self))]
    async fn group_get(
        &mut self,
        room: SignalingRoomId,
    ) -> Result<Option<String>, SignalingModuleError> {
        Ok(state().read().group_get(room))
    }

    #[tracing::instrument(name = "delete_protocol_group", skip(self))]
    async fn group_delete(&mut self, room: SignalingRoomId) -> Result<(), SignalingModuleError> {
        state().write().group_delete(room);
        Ok(())
    }

    #[tracing::instrument(name = "protocol_try_start_init", skip(self))]
    async fn try_start_init(
        &mut self,
        room: SignalingRoomId,
    ) -> Result<Option<InitState>, SignalingModuleError> {
        Ok(state().write().init_get_or_default(room))
    }
}

#[cfg(test)]
mod test {
    use opentalk_signaling_core::VolatileStaticMemoryStorage;
    use serial_test::serial;

    use super::{super::super::test_common, state};

    fn storage() -> VolatileStaticMemoryStorage {
        state().write().reset();
        VolatileStaticMemoryStorage
    }

    #[tokio::test]
    #[serial]
    async fn group() {
        test_common::group(&mut storage()).await;
    }

    #[tokio::test]
    #[serial]
    async fn init() {
        test_common::init(&mut storage()).await;
    }
}
