// SPDX-FileCopyrightText: OpenTalk GmbH <mail@opentalk.eu>
//
// SPDX-License-Identifier: EUPL-1.2

use std::{
    collections::BTreeMap,
    sync::{Arc, OnceLock},
};

use async_trait::async_trait;
use opentalk_signaling_core::{SignalingModuleError, SignalingRoomId, VolatileStaticMemoryStorage};
use opentalk_types_signaling::ParticipantId;
use opentalk_types_signaling_subroom_audio::{state::WhisperState, whisper_id::WhisperId};
use parking_lot::RwLock;

use super::memory::MemorySubroomAudio;
use crate::storage::SubroomAudioStorage;

static STATE: OnceLock<Arc<RwLock<MemorySubroomAudio>>> = OnceLock::new();

fn state() -> &'static Arc<RwLock<MemorySubroomAudio>> {
    STATE.get_or_init(Default::default)
}

#[async_trait(?Send)]
impl SubroomAudioStorage for VolatileStaticMemoryStorage {
    #[tracing::instrument(level = "debug", skip(self))]
    async fn create_whisper_group(
        &mut self,
        room: SignalingRoomId,
        whisper_id: WhisperId,
        participants: &BTreeMap<ParticipantId, WhisperState>,
    ) -> Result<(), SignalingModuleError> {
        state()
            .write()
            .create_whisper_group(room, whisper_id, participants)
    }

    #[tracing::instrument(level = "debug", skip(self))]
    async fn get_whisper_group(
        &mut self,
        room: SignalingRoomId,
        whisper_id: WhisperId,
    ) -> Result<BTreeMap<ParticipantId, WhisperState>, SignalingModuleError> {
        state().read().get_whisper_group(room, whisper_id)
    }

    #[tracing::instrument(level = "debug", skip(self))]
    async fn get_all_whisper_group_ids(
        &mut self,
        room: SignalingRoomId,
    ) -> Result<Vec<WhisperId>, SignalingModuleError> {
        Ok(state().read().get_whisper_group_ids(room))
    }

    #[tracing::instrument(level = "debug", skip(self))]
    async fn delete_whisper_group(
        &mut self,
        room: SignalingRoomId,
        whisper_id: WhisperId,
    ) -> Result<(), SignalingModuleError> {
        state().write().remove_whisper_group(room, whisper_id);
        Ok(())
    }

    #[tracing::instrument(level = "debug", skip(self))]
    async fn add_participants(
        &mut self,
        room: SignalingRoomId,
        whisper_id: WhisperId,
        participant_ids: &BTreeMap<ParticipantId, WhisperState>,
    ) -> Result<(), SignalingModuleError> {
        state()
            .write()
            .add_participants(room, whisper_id, participant_ids);
        Ok(())
    }

    #[tracing::instrument(level = "debug", skip(self))]
    async fn remove_participant(
        &mut self,
        room: SignalingRoomId,
        whisper_id: WhisperId,
        participant_id: ParticipantId,
    ) -> Result<bool, SignalingModuleError> {
        state()
            .write()
            .remove_participant(room, whisper_id, participant_id)
    }

    #[tracing::instrument(level = "debug", skip(self))]
    async fn update_participant_state(
        &mut self,
        room: SignalingRoomId,
        whisper_id: WhisperId,
        participant_id: ParticipantId,
        new_state: WhisperState,
    ) -> Result<(), SignalingModuleError> {
        state()
            .write()
            .update_participant_state(room, whisper_id, participant_id, new_state)
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
    async fn create_group() {
        test_common::create_group(&mut storage()).await
    }

    #[tokio::test]
    #[serial]
    async fn add_participant() {
        test_common::add_participant(&mut storage()).await
    }

    #[tokio::test]
    #[serial]
    async fn update_participant() {
        test_common::update_participant(&mut storage()).await
    }

    #[tokio::test]
    #[serial]
    async fn remove_participant() {
        test_common::remove_participant(&mut storage()).await
    }

    #[tokio::test]
    #[serial]
    async fn manage_groups() {
        test_common::manage_groups(&mut storage()).await
    }
}
