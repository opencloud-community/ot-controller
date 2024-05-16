// SPDX-FileCopyrightText: OpenTalk GmbH <mail@opentalk.eu>
//
// SPDX-License-Identifier: EUPL-1.2

use std::collections::BTreeSet;

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
    ) -> Result<BTreeSet<ParticipantId>, SignalingModuleError>;

    async fn remove_participant_set(
        &mut self,
        room: SignalingRoomId,
    ) -> Result<(), SignalingModuleError>;

    async fn participants_contains(
        &mut self,
        room: SignalingRoomId,
        participant: ParticipantId,
    ) -> Result<bool, SignalingModuleError>;

    async fn check_participants_exist(
        &mut self,
        room: SignalingRoomId,
        participants: &[ParticipantId],
    ) -> Result<bool, SignalingModuleError>;

    /// Returns `true` if the participant id was added, `false` if it already were present
    async fn add_participant_to_set(
        &mut self,
        room: SignalingRoomId,
        participant: ParticipantId,
    ) -> Result<bool, SignalingModuleError>;

    async fn get_attribute<V>(
        &mut self,
        room: SignalingRoomId,
        participant: ParticipantId,
        name: &str,
    ) -> Result<V, SignalingModuleError>
    where
        V: redis::FromRedisValue;

    async fn set_attribute<V>(
        &mut self,
        room: SignalingRoomId,
        participant: ParticipantId,
        name: &str,
        value: V,
    ) -> Result<(), SignalingModuleError>
    where
        V: core::fmt::Debug + redis::ToRedisArgs + Send + Sync;

    async fn remove_attribute(
        &mut self,
        room: SignalingRoomId,
        participant: ParticipantId,
        name: &str,
    ) -> Result<(), SignalingModuleError>;

    /// Get attribute values for multiple participants
    ///
    /// The index of the attributes in the returned vector is a direct mapping to the provided list of participants.
    async fn get_attribute_for_participants<V>(
        &mut self,
        room: SignalingRoomId,
        name: &str,
        participants: &[ParticipantId],
    ) -> Result<Vec<Option<V>>, SignalingModuleError>
    where
        V: redis::FromRedisValue;
}
