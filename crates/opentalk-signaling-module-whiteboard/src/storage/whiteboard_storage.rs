// SPDX-FileCopyrightText: OpenTalk GmbH <mail@opentalk.eu>
//
// SPDX-License-Identifier: EUPL-1.2

use async_trait::async_trait;
use opentalk_signaling_core::{SignalingModuleError, SignalingRoomId};

use super::{InitState, SpaceInfo};

#[async_trait(?Send)]
pub(crate) trait WhiteboardStorage {
    /// Attempts to set the spacedeck state to [`InitState::Initializing`] with a SETNX command.
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

    /// Sets the room state to [`InitState::Initialized(..)`]
    async fn set_initialized(
        &mut self,
        room_id: SignalingRoomId,
        space_info: SpaceInfo,
    ) -> Result<(), SignalingModuleError>;

    async fn get_init_state(
        &mut self,
        room_id: SignalingRoomId,
    ) -> Result<Option<InitState>, SignalingModuleError>;

    async fn delete_init_state(
        &mut self,
        room_id: SignalingRoomId,
    ) -> Result<(), SignalingModuleError>;
}
