// SPDX-FileCopyrightText: OpenTalk GmbH <mail@opentalk.eu>
//
// SPDX-License-Identifier: EUPL-1.2
use std::sync::{Arc, OnceLock};

use async_trait::async_trait;
use opentalk_signaling_core::{SignalingModuleError, SignalingRoomId, VolatileStaticMemoryStorage};
use parking_lot::RwLock;

use super::memory::MemoryWhiteboardState;
use crate::storage::{InitState, WhiteboardStorage};

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
    async fn initialization() {
        test_common::initialization(&mut storage()).await;
    }
}
