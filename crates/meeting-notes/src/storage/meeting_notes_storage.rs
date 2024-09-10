// SPDX-FileCopyrightText: OpenTalk GmbH <mail@opentalk.eu>
//
// SPDX-License-Identifier: EUPL-1.2

use async_trait::async_trait;
use opentalk_signaling_core::{
    control::storage::{ControlStorageParticipantAttributesRaw, ControlStorageParticipantSet},
    SignalingModuleError, SignalingRoomId,
};
use opentalk_types::core::ParticipantId;

use super::InitState;
use crate::SessionInfo;

#[async_trait(?Send)]
pub(crate) trait MeetingNotesStorage:
    ControlStorageParticipantSet + ControlStorageParticipantAttributesRaw
{
    async fn group_set(
        &mut self,
        room_id: SignalingRoomId,
        group_id: &str,
    ) -> Result<(), SignalingModuleError>;

    async fn group_get(
        &mut self,
        room_id: SignalingRoomId,
    ) -> Result<Option<String>, SignalingModuleError>;

    async fn group_delete(&mut self, room_id: SignalingRoomId) -> Result<(), SignalingModuleError>;

    /// Attempts to set the room state to [`InitState::Initializing`] with a SETNX command.
    ///
    /// If the key already holds a value, the current key gets returned without changing the state.
    ///
    /// Behaves like a SETNX-GET redis command.
    ///
    /// When the key was empty and the `Initializing` state was set, Ok(None) will be returned.
    async fn try_start_init(
        &mut self,
        room_id: SignalingRoomId,
    ) -> Result<Option<InitState>, SignalingModuleError>;

    /// Sets the room state to [`InitState::Initialized`]
    async fn set_initialized(&mut self, room: SignalingRoomId) -> Result<(), SignalingModuleError>;

    async fn init_get(
        &mut self,
        room: SignalingRoomId,
    ) -> Result<Option<InitState>, SignalingModuleError>;

    async fn init_delete(&mut self, room: SignalingRoomId) -> Result<(), SignalingModuleError>;

    async fn session_get(
        &mut self,
        room: SignalingRoomId,
        participant: ParticipantId,
    ) -> Result<Option<SessionInfo>, SignalingModuleError>;

    async fn session_set(
        &mut self,
        room: SignalingRoomId,
        participant: ParticipantId,
        session_info: &SessionInfo,
    ) -> Result<(), SignalingModuleError>;

    async fn session_delete(
        &mut self,
        room_id: SignalingRoomId,
        participant_id: ParticipantId,
    ) -> Result<Option<SessionInfo>, SignalingModuleError>;

    /// Remove all redis keys related to this room & module
    #[tracing::instrument(name = "cleanup_meeting_notes", skip(self))]
    async fn cleanup(&mut self, room: SignalingRoomId) -> Result<(), SignalingModuleError> {
        self.init_delete(room).await?;
        self.group_delete(room).await?;

        Ok(())
    }
}
