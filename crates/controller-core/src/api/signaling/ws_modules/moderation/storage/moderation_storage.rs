// SPDX-FileCopyrightText: OpenTalk GmbH <mail@opentalk.eu>
//
// SPDX-License-Identifier: EUPL-1.2

use std::collections::BTreeSet;

use async_trait::async_trait;
use opentalk_signaling_core::{
    control::storage::{
        ControlStorageParticipantAttributesRaw, ControlStorageParticipantSet,
        ControlStorageSkipWaitingRoom,
    },
    SignalingModuleError,
};
use opentalk_types_common::{rooms::RoomId, users::UserId};
use opentalk_types_signaling::ParticipantId;

#[async_trait(?Send)]
pub(crate) trait ModerationStorage:
    ControlStorageParticipantAttributesRaw
    + ControlStorageSkipWaitingRoom
    + ControlStorageParticipantSet
{
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

    async fn is_raise_hands_enabled(&mut self, room: RoomId) -> Result<bool, SignalingModuleError>;

    async fn delete_raise_hands_enabled(
        &mut self,
        room: RoomId,
    ) -> Result<(), SignalingModuleError>;

    /// Add a participant to the waiting room.
    ///
    /// Returns `Ok(true)` if the participant was added, `Ok(false)` if the
    /// participant already was in the waiting room before.
    async fn waiting_room_add_participant(
        &mut self,
        room: RoomId,
        participant: ParticipantId,
    ) -> Result<bool, SignalingModuleError>;

    async fn waiting_room_remove_participant(
        &mut self,
        room: RoomId,
        participant: ParticipantId,
    ) -> Result<(), SignalingModuleError>;

    async fn waiting_room_contains_participant(
        &mut self,
        room: RoomId,
        participant: ParticipantId,
    ) -> Result<bool, SignalingModuleError>;

    async fn waiting_room_participants(
        &mut self,
        room: RoomId,
    ) -> Result<BTreeSet<ParticipantId>, SignalingModuleError>;

    #[cfg(test)]
    async fn waiting_room_participant_count(
        &mut self,
        room: RoomId,
    ) -> Result<usize, SignalingModuleError>;

    async fn delete_waiting_room(&mut self, room: RoomId) -> Result<(), SignalingModuleError>;

    /// Add a participant to the waiting room accepted list.
    ///
    /// Returns `Ok(true)` if the participant was added, `Ok(false)` if the
    /// participant already was in the waiting room accepted list before.
    async fn waiting_room_accepted_add_participant(
        &mut self,
        room: RoomId,
        participant: ParticipantId,
    ) -> Result<bool, SignalingModuleError>;

    async fn waiting_room_accepted_remove_participant(
        &mut self,
        room: RoomId,
        participant: ParticipantId,
    ) -> Result<(), SignalingModuleError>;

    async fn waiting_room_accepted_remove_participants(
        &mut self,
        room: RoomId,
        participants: &[ParticipantId],
    ) -> Result<(), SignalingModuleError>;

    async fn waiting_room_accepted_participants(
        &mut self,
        room: RoomId,
    ) -> Result<BTreeSet<ParticipantId>, SignalingModuleError>;

    #[cfg(test)]
    async fn waiting_room_accepted_participant_count(
        &mut self,
        room: RoomId,
    ) -> Result<usize, SignalingModuleError>;

    async fn delete_waiting_room_accepted(
        &mut self,
        room: RoomId,
    ) -> Result<(), SignalingModuleError>;
}
