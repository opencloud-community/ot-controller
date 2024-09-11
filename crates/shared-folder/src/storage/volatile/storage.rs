// SPDX-FileCopyrightText: OpenTalk GmbH <mail@opentalk.eu>
//
// SPDX-License-Identifier: EUPL-1.2
use std::sync::{Arc, OnceLock};

use async_trait::async_trait;
use opentalk_signaling_core::{SignalingModuleError, SignalingRoomId, VolatileStaticMemoryStorage};
use opentalk_types_common::shared_folders::SharedFolder;
use parking_lot::RwLock;

use super::memory::MemorySharedFolderState;
use crate::storage::SharedFolderStorage;

static STATE: OnceLock<Arc<RwLock<MemorySharedFolderState>>> = OnceLock::new();

fn state() -> &'static Arc<RwLock<MemorySharedFolderState>> {
    STATE.get_or_init(Default::default)
}

#[async_trait(?Send)]
impl SharedFolderStorage for VolatileStaticMemoryStorage {
    #[tracing::instrument(level = "debug", skip(self))]
    async fn set_shared_folder_initialized(
        &mut self,
        room: SignalingRoomId,
    ) -> Result<(), SignalingModuleError> {
        state().write().set_shared_folder_initialized(room);
        Ok(())
    }

    #[tracing::instrument(level = "debug", skip(self))]
    async fn is_shared_folder_initialized(
        &mut self,
        room: SignalingRoomId,
    ) -> Result<bool, SignalingModuleError> {
        Ok(state().read().is_shared_folder_initialized(room))
    }

    #[tracing::instrument(level = "debug", skip(self))]
    async fn delete_shared_folder_initialized(
        &mut self,
        room: SignalingRoomId,
    ) -> Result<(), SignalingModuleError> {
        state().write().delete_shared_folder_initialized(room);
        Ok(())
    }

    #[tracing::instrument(level = "debug", skip(self))]
    async fn get_shared_folder(
        &mut self,
        room: SignalingRoomId,
    ) -> Result<Option<SharedFolder>, SignalingModuleError> {
        Ok(state().read().get_shared_folder(room))
    }

    #[tracing::instrument(level = "debug", skip(self))]
    async fn set_shared_folder(
        &mut self,
        room: SignalingRoomId,
        value: SharedFolder,
    ) -> Result<(), SignalingModuleError> {
        state().write().set_shared_folder(room, value);
        Ok(())
    }

    #[tracing::instrument(level = "debug", skip(self))]
    async fn delete_shared_folder(
        &mut self,
        room: SignalingRoomId,
    ) -> Result<(), SignalingModuleError> {
        state().write().delete_shared_folder(room);
        Ok(())
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
    async fn initialized() {
        test_common::initialized(&mut storage()).await;
    }

    #[tokio::test]
    #[serial]
    async fn shared_folder() {
        test_common::shared_folder(&mut storage()).await;
    }
}
