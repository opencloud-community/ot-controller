// SPDX-FileCopyrightText: OpenTalk GmbH <mail@opentalk.eu>
//
// SPDX-License-Identifier: EUPL-1.2

use async_trait::async_trait;
use opentalk_signaling_core::{SignalingModuleError, SignalingRoomId};

use super::InitState;

#[async_trait(?Send)]
pub(crate) trait ProtocolStorage {
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
}
