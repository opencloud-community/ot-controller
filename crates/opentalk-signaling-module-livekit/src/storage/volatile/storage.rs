// SPDX-FileCopyrightText: OpenTalk GmbH <mail@opentalk.eu>
//
// SPDX-License-Identifier: EUPL-1.2

use std::sync::{Arc, OnceLock};

use async_trait::async_trait;
use opentalk_signaling_core::{SignalingModuleError, VolatileStaticMemoryStorage};
use opentalk_types_common::rooms::RoomId;
use opentalk_types_signaling::ParticipantId;
use opentalk_types_signaling_livekit::MicrophoneRestrictionState;
use parking_lot::RwLock;

use super::memory::MemoryLivekitState;
use crate::storage::livekit_storage::LivekitStorage;

static STATE: OnceLock<Arc<RwLock<MemoryLivekitState>>> = OnceLock::new();

fn state() -> &'static Arc<RwLock<MemoryLivekitState>> {
    STATE.get_or_init(Default::default)
}

#[async_trait]
impl LivekitStorage for VolatileStaticMemoryStorage {
    #[tracing::instrument(level = "debug", skip(self))]
    async fn set_microphone_restriction_allow_list(
        &mut self,
        room: RoomId,
        participants: &[ParticipantId],
    ) -> Result<(), SignalingModuleError> {
        state()
            .write()
            .force_mute_set_allow_unmute(room, participants);
        Ok(())
    }

    #[tracing::instrument(level = "debug", skip(self))]
    async fn clear_microphone_restriction(
        &mut self,
        room: RoomId,
    ) -> Result<(), SignalingModuleError> {
        state().write().clear_force_mute(room);
        Ok(())
    }

    async fn get_microphone_restriction_state(
        &mut self,
        room: RoomId,
    ) -> Result<MicrophoneRestrictionState, SignalingModuleError> {
        Ok(state().read().get_force_mute_state(room))
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
    async fn force_mute() {
        test_common::force_mute(&mut storage().await).await;
    }
}
