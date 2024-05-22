// SPDX-FileCopyrightText: OpenTalk GmbH <mail@opentalk.eu>
//
// SPDX-License-Identifier: EUPL-1.2

use std::sync::{Arc, OnceLock};

use async_trait::async_trait;
use opentalk_signaling_core::{SignalingModuleError, SignalingRoomId, VolatileStaticMemoryStorage};
use opentalk_types::core::ParticipantId;
use parking_lot::RwLock;

use super::memory::MemoryTimerState;
use crate::storage::timer_storage::TimerStorage;

static STATE: OnceLock<Arc<RwLock<MemoryTimerState>>> = OnceLock::new();

fn state() -> &'static Arc<RwLock<MemoryTimerState>> {
    STATE.get_or_init(Default::default)
}

#[async_trait(?Send)]
impl TimerStorage for VolatileStaticMemoryStorage {
    #[tracing::instrument(name = "meeting_timer_ready_set", skip(self))]
    async fn ready_status_set(
        &mut self,
        room_id: SignalingRoomId,
        participant_id: ParticipantId,
        ready_status: bool,
    ) -> Result<(), SignalingModuleError> {
        state()
            .write()
            .ready_status_set(room_id, participant_id, ready_status);
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
    async fn ready_status() {
        test_common::ready_status(&mut storage()).await;
    }
}
