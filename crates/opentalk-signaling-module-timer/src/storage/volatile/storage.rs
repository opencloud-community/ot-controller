// SPDX-FileCopyrightText: OpenTalk GmbH <mail@opentalk.eu>
//
// SPDX-License-Identifier: EUPL-1.2

use std::sync::{Arc, OnceLock};

use async_trait::async_trait;
use opentalk_signaling_core::{SignalingModuleError, SignalingRoomId, VolatileStaticMemoryStorage};
use opentalk_types_signaling::ParticipantId;
use opentalk_types_signaling_timer::peer_state::TimerPeerState;
use parking_lot::RwLock;

use super::memory::MemoryTimerState;
use crate::storage::{Timer, timer_storage::TimerStorage};

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

    #[tracing::instrument(name = "meeting_timer_ready_get", skip(self))]
    async fn ready_status_get(
        &mut self,
        room: SignalingRoomId,
        participant: ParticipantId,
    ) -> Result<Option<TimerPeerState>, SignalingModuleError> {
        Ok(state().read().ready_status_get(room, participant))
    }

    #[tracing::instrument(name = "meeting_timer_ready_delete", skip(self))]
    async fn ready_status_delete(
        &mut self,
        room: SignalingRoomId,
        participant: ParticipantId,
    ) -> Result<(), SignalingModuleError> {
        state().write().ready_status_delete(room, participant);
        Ok(())
    }

    #[tracing::instrument(name = "meeting_timer_set", skip(self, timer))]
    async fn timer_set_if_not_exists(
        &mut self,
        room: SignalingRoomId,
        timer: &Timer,
    ) -> Result<bool, SignalingModuleError> {
        Ok(state().write().timer_set_if_not_exists(room, timer.clone()))
    }

    #[tracing::instrument(name = "meeting_timer_get", skip(self))]
    async fn timer_get(
        &mut self,
        room: SignalingRoomId,
    ) -> Result<Option<Timer>, SignalingModuleError> {
        Ok(state().read().timer_get(room))
    }

    #[tracing::instrument(name = "meeting_timer_delete", skip(self))]
    async fn timer_delete(
        &mut self,
        room: SignalingRoomId,
    ) -> Result<Option<Timer>, SignalingModuleError> {
        Ok(state().write().timer_delete(room))
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
    async fn ready_status() {
        test_common::ready_status(&mut storage()).await;
    }

    #[tokio::test]
    #[serial]
    async fn timer() {
        test_common::timer(&mut storage()).await;
    }
}
