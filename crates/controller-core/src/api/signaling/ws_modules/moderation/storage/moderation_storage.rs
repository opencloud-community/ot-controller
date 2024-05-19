// SPDX-FileCopyrightText: OpenTalk GmbH <mail@opentalk.eu>
//
// SPDX-License-Identifier: EUPL-1.2

use async_trait::async_trait;
use opentalk_signaling_core::SignalingModuleError;
use opentalk_types::core::{RoomId, UserId};

#[async_trait(?Send)]
pub(crate) trait ModerationStorage {
    async fn ban_user(&mut self, room: RoomId, user: UserId) -> Result<(), SignalingModuleError>;

    async fn is_user_banned(
        &mut self,
        room: RoomId,
        user: UserId,
    ) -> Result<bool, SignalingModuleError>;

    async fn delete_user_bans(&mut self, room: RoomId) -> Result<(), SignalingModuleError>;

    /// Return the `waiting_room` flag, and optionally set it to a defined value
    /// given by the `enabled` parameter beforehand if the flag is not present yet.
    async fn init_waiting_room_enabled(
        &mut self,
        room: RoomId,
        enabled: bool,
    ) -> Result<bool, SignalingModuleError>;

    async fn set_waiting_room_enabled(
        &mut self,
        room: RoomId,
        enabled: bool,
    ) -> Result<(), SignalingModuleError>;

    async fn is_waiting_room_enabled(&mut self, room: RoomId)
        -> Result<bool, SignalingModuleError>;

    async fn delete_waiting_room_enabled(
        &mut self,
        room: RoomId,
    ) -> Result<(), SignalingModuleError>;

    async fn set_raise_hands_enabled(
        &mut self,
        room: RoomId,
        enabled: bool,
    ) -> Result<(), SignalingModuleError>;
}
