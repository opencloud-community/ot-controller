// SPDX-FileCopyrightText: OpenTalk GmbH <mail@opentalk.eu>
//
// SPDX-License-Identifier: EUPL-1.2

use async_trait::async_trait;
use opentalk_signaling_core::{
    control::storage::ControlStorageParticipantSet, SignalingModuleError, SignalingRoomId,
};
use opentalk_types::{
    core::{ParticipantId, Timestamp},
    signaling::media::{ParticipantMediaState, ParticipantSpeakingState, SpeakingState},
};

use crate::mcu::{McuId, MediaSessionKey, PublisherInfo};

#[async_trait]
pub(crate) trait MediaStorage: ControlStorageParticipantSet + Send {
    async fn get_media_state(
        &mut self,
        room: SignalingRoomId,
        participant: ParticipantId,
    ) -> Result<Option<ParticipantMediaState>, SignalingModuleError>;

    async fn set_media_state(
        &mut self,
        room: SignalingRoomId,
        participant: ParticipantId,
        participant_media_state: &ParticipantMediaState,
    ) -> Result<(), SignalingModuleError>;

    async fn delete_media_state(
        &mut self,
        room: SignalingRoomId,
        participant: ParticipantId,
    ) -> Result<(), SignalingModuleError>;

    async fn add_presenter(
        &mut self,
        room: SignalingRoomId,
        participant: ParticipantId,
    ) -> Result<(), SignalingModuleError>;

    async fn remove_presenter(
        &mut self,
        room: SignalingRoomId,
        participant: ParticipantId,
    ) -> Result<(), SignalingModuleError>;

    async fn is_presenter(
        &mut self,
        room: SignalingRoomId,
        participant: ParticipantId,
    ) -> Result<bool, SignalingModuleError>;

    async fn clear_presenters(&mut self, room: SignalingRoomId)
        -> Result<(), SignalingModuleError>;

    async fn set_speaking_state(
        &mut self,
        room: SignalingRoomId,
        participant: ParticipantId,
        is_speaking: bool,
        updated_at: Timestamp,
    ) -> Result<(), SignalingModuleError>;

    async fn get_speaking_state(
        &mut self,
        room: SignalingRoomId,
        participant: ParticipantId,
    ) -> Result<Option<SpeakingState>, SignalingModuleError>;

    async fn delete_speaking_state(
        &mut self,
        room: SignalingRoomId,
        participant: ParticipantId,
    ) -> Result<(), SignalingModuleError>;

    async fn delete_speaking_state_multiple_participants(
        &mut self,
        room: SignalingRoomId,
        participants: &[ParticipantId],
    ) -> Result<(), SignalingModuleError>;

    async fn get_speaking_state_multiple_participants(
        &mut self,
        room: SignalingRoomId,
        participants: &[ParticipantId],
    ) -> Result<Vec<ParticipantSpeakingState>, SignalingModuleError>;

    async fn initialize_mcu_load(
        &mut self,
        mcu_id: McuId,
        index: Option<usize>,
    ) -> Result<(), SignalingModuleError>;

    async fn get_mcus_sorted_by_load(
        &mut self,
    ) -> Result<Vec<(McuId, Option<usize>)>, SignalingModuleError>;

    async fn increase_mcu_load(
        &mut self,
        mcu_id: McuId,
        index: Option<usize>,
    ) -> Result<(), SignalingModuleError>;

    async fn decrease_mcu_load(
        &mut self,
        mcu_id: McuId,
        index: Option<usize>,
    ) -> Result<(), SignalingModuleError>;

    async fn set_publisher_info(
        &mut self,
        media_session_key: MediaSessionKey,
        info: PublisherInfo,
    ) -> Result<(), SignalingModuleError>;

    async fn get_publisher_info(
        &mut self,
        media_session_key: MediaSessionKey,
    ) -> Result<PublisherInfo, SignalingModuleError>;

    async fn delete_publisher_info(
        &mut self,
        key: MediaSessionKey,
    ) -> Result<(), redis::RedisError>;
}
