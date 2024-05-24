// SPDX-FileCopyrightText: OpenTalk GmbH <mail@opentalk.eu>
//
// SPDX-License-Identifier: EUPL-1.2

use std::sync::{Arc, OnceLock};

use async_trait::async_trait;
use opentalk_signaling_core::{SignalingModuleError, SignalingRoomId, VolatileStaticMemoryStorage};
use opentalk_types::{
    core::{ParticipantId, Timestamp},
    signaling::media::{ParticipantMediaState, ParticipantSpeakingState, SpeakingState},
};
use parking_lot::RwLock;

use super::memory::MemoryMediaState;
use crate::{mcu::McuId, storage::media_storage::MediaStorage};

static STATE: OnceLock<Arc<RwLock<MemoryMediaState>>> = OnceLock::new();

fn state() -> &'static Arc<RwLock<MemoryMediaState>> {
    STATE.get_or_init(Default::default)
}

#[async_trait]
impl MediaStorage for VolatileStaticMemoryStorage {
    #[tracing::instrument(level = "debug", skip(self))]
    async fn get_media_state(
        &mut self,
        room: SignalingRoomId,
        participant: ParticipantId,
    ) -> Result<Option<ParticipantMediaState>, SignalingModuleError> {
        Ok(state().read().get_media_state(room, participant))
    }

    #[tracing::instrument(level = "debug", skip(self))]
    async fn set_media_state(
        &mut self,
        room: SignalingRoomId,
        participant: ParticipantId,
        participant_media_state: &ParticipantMediaState,
    ) -> Result<(), SignalingModuleError> {
        state()
            .write()
            .set_media_state(room, participant, participant_media_state);
        Ok(())
    }

    #[tracing::instrument(level = "debug", skip(self))]
    async fn delete_media_state(
        &mut self,
        room: SignalingRoomId,
        participant: ParticipantId,
    ) -> Result<(), SignalingModuleError> {
        state().write().delete_media_state(room, participant);
        Ok(())
    }

    #[tracing::instrument(level = "debug", skip(self))]
    async fn add_presenter(
        &mut self,
        room: SignalingRoomId,
        participant: ParticipantId,
    ) -> Result<(), SignalingModuleError> {
        state().write().add_presenter(room, participant);
        Ok(())
    }

    #[tracing::instrument(level = "debug", skip(self))]
    async fn is_presenter(
        &mut self,
        room: SignalingRoomId,
        participant: ParticipantId,
    ) -> Result<bool, SignalingModuleError> {
        Ok(state().read().is_presenter(room, participant))
    }

    #[tracing::instrument(level = "debug", skip(self))]
    async fn remove_presenter(
        &mut self,
        room: SignalingRoomId,
        participant: ParticipantId,
    ) -> Result<(), SignalingModuleError> {
        state().write().remove_presenter(room, participant);
        Ok(())
    }

    #[tracing::instrument(level = "debug", skip(self))]
    async fn clear_presenters(
        &mut self,
        room: SignalingRoomId,
    ) -> Result<(), SignalingModuleError> {
        state().write().clear_presenters(room);
        Ok(())
    }

    #[tracing::instrument(level = "debug", skip(self))]
    async fn set_speaking_state(
        &mut self,
        room: SignalingRoomId,
        participant: ParticipantId,
        is_speaking: bool,
        updated_at: Timestamp,
    ) -> Result<(), SignalingModuleError> {
        state()
            .write()
            .set_speaking_state(room, participant, is_speaking, updated_at);
        Ok(())
    }

    #[tracing::instrument(level = "debug", skip(self))]
    async fn get_speaking_state(
        &mut self,
        room: SignalingRoomId,
        participant: ParticipantId,
    ) -> Result<Option<SpeakingState>, SignalingModuleError> {
        Ok(state().read().get_speaking_state(room, participant))
    }

    #[tracing::instrument(level = "debug", skip(self))]
    async fn delete_speaking_state(
        &mut self,
        room: SignalingRoomId,
        participant: ParticipantId,
    ) -> Result<(), SignalingModuleError> {
        state().write().delete_speaking_state(room, participant);
        Ok(())
    }

    #[tracing::instrument(level = "debug", skip(self))]
    async fn delete_speaking_state_multiple_participants(
        &mut self,
        room: SignalingRoomId,
        participants: &[ParticipantId],
    ) -> Result<(), SignalingModuleError> {
        state()
            .write()
            .delete_speaking_state_multiple_participants(room, participants);
        Ok(())
    }

    #[tracing::instrument(level = "debug", skip(self))]
    async fn get_speaking_state_multiple_participants(
        &mut self,
        room: SignalingRoomId,
        participants: &[ParticipantId],
    ) -> Result<Vec<ParticipantSpeakingState>, SignalingModuleError> {
        Ok(state()
            .read()
            .get_speaking_state_multiple_participants(room, participants))
    }

    #[tracing::instrument(level = "debug", skip(self))]
    async fn initialize_mcu_load(
        &mut self,
        mcu_id: McuId,
        index: Option<usize>,
    ) -> Result<(), SignalingModuleError> {
        state().write().initialize_mcu_load(mcu_id, index);
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
    async fn media_state() {
        test_common::media_state(&mut storage().await).await;
    }

    #[tokio::test]
    #[serial]
    async fn presenter() {
        test_common::presenter(&mut storage().await).await;
    }

    #[tokio::test]
    #[serial]
    async fn speaking_state() {
        test_common::speaking_state(&mut storage().await).await;
    }

    #[tokio::test]
    #[serial]
    async fn mcu_load() {
        test_common::mcu_load(&mut storage().await).await;
    }
}
