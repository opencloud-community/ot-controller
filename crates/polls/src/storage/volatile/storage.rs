// SPDX-FileCopyrightText: OpenTalk GmbH <mail@opentalk.eu>
//
// SPDX-License-Identifier: EUPL-1.2

use std::sync::{Arc, OnceLock};

use async_trait::async_trait;
use opentalk_signaling_core::{SignalingModuleError, SignalingRoomId, VolatileStaticMemoryStorage};
use opentalk_types::signaling::polls::state::PollsState;
use parking_lot::RwLock;

use super::memory::MemoryPollsState;
use crate::storage::polls_storage::PollsStorage;

static STATE: OnceLock<Arc<RwLock<MemoryPollsState>>> = OnceLock::new();

fn state() -> &'static Arc<RwLock<MemoryPollsState>> {
    STATE.get_or_init(Default::default)
}

#[async_trait(?Send)]
impl PollsStorage for VolatileStaticMemoryStorage {
    #[tracing::instrument(level = "debug", skip(self))]
    async fn get_polls_state(
        &mut self,
        room: SignalingRoomId,
    ) -> Result<Option<PollsState>, SignalingModuleError> {
        Ok(state().read().get_polls_state(room))
    }

    #[tracing::instrument(level = "debug", skip(self))]
    async fn set_polls_state(
        &mut self,
        room: SignalingRoomId,
        polls_state: &PollsState,
    ) -> Result<bool, SignalingModuleError> {
        Ok(state().write().set_polls_state(room, polls_state))
    }

    #[tracing::instrument(level = "debug", skip(self))]
    async fn delete_polls_state(
        &mut self,
        room: SignalingRoomId,
    ) -> Result<(), SignalingModuleError> {
        state().write().delete_polls_state(&room);
        Ok(())
    }
}

#[cfg(test)]
mod test {
    use opentalk_signaling_core::VolatileStaticMemoryStorage;
    use serial_test::serial;

    use super::{super::super::test_common, state};

    async fn storage() -> VolatileStaticMemoryStorage {
        state().write().reset();
        VolatileStaticMemoryStorage
    }

    #[tokio::test]
    #[serial]
    async fn polls_state() {
        test_common::polls_state(&mut storage().await).await;
    }
}
