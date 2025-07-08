// SPDX-FileCopyrightText: OpenTalk GmbH <mail@opentalk.eu>
//
// SPDX-License-Identifier: EUPL-1.2

use std::collections::BTreeMap;

use async_trait::async_trait;
use opentalk_signaling_core::{
    SignalingModuleError, SignalingRoomId, control::storage::ControlStorageParticipantSet,
};
use opentalk_types_signaling::ParticipantId;
use opentalk_types_signaling_subroom_audio::{state::WhisperState, whisper_id::WhisperId};

#[async_trait(?Send)]
pub(crate) trait SubroomAudioStorage: ControlStorageParticipantSet {
    async fn create_whisper_group(
        &mut self,
        room: SignalingRoomId,
        whisper_id: WhisperId,
        participants: &BTreeMap<ParticipantId, WhisperState>,
    ) -> Result<(), SignalingModuleError>;

    async fn get_whisper_group(
        &mut self,
        room: SignalingRoomId,
        whisper_id: WhisperId,
    ) -> Result<BTreeMap<ParticipantId, WhisperState>, SignalingModuleError>;

    async fn get_all_whisper_group_ids(
        &mut self,
        room: SignalingRoomId,
    ) -> Result<Vec<WhisperId>, SignalingModuleError>;

    async fn delete_whisper_group(
        &mut self,
        room: SignalingRoomId,
        whisper_id: WhisperId,
    ) -> Result<(), SignalingModuleError>;

    async fn add_participants(
        &mut self,
        room: SignalingRoomId,
        whisper_id: WhisperId,
        participant_ids: &BTreeMap<ParticipantId, WhisperState>,
    ) -> Result<(), SignalingModuleError>;

    async fn remove_participant(
        &mut self,
        room: SignalingRoomId,
        whisper_id: WhisperId,
        participant_id: ParticipantId,
    ) -> Result<bool, SignalingModuleError>;

    async fn update_participant_state(
        &mut self,
        room: SignalingRoomId,
        whisper_id: WhisperId,
        participant_id: ParticipantId,
        state: WhisperState,
    ) -> Result<(), SignalingModuleError>;
}
