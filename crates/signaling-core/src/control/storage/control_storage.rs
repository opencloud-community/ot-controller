// SPDX-FileCopyrightText: OpenTalk GmbH <mail@opentalk.eu>
//
// SPDX-License-Identifier: EUPL-1.2

use async_trait::async_trait;
use opentalk_types::core::ParticipantId;

use crate::{SignalingModuleError, SignalingRoomId};

#[async_trait(?Send)]
pub trait ControlStorage {
    async fn participant_set_exists(
        &mut self,
        room: SignalingRoomId,
    ) -> Result<bool, SignalingModuleError>;

    async fn get_all_participants(
        &mut self,
        room: SignalingRoomId,
    ) -> Result<Vec<ParticipantId>, SignalingModuleError>;

    async fn remove_participant_set(
        &mut self,
        room: SignalingRoomId,
    ) -> Result<(), SignalingModuleError>;
}
