// SPDX-FileCopyrightText: OpenTalk GmbH <mail@opentalk.eu>
//
// SPDX-License-Identifier: EUPL-1.2

use std::collections::{BTreeMap, BTreeSet};

use async_trait::async_trait;
use opentalk_db_storage::{events::Event, tariffs::Tariff};
use opentalk_types::{
    core::{ParticipantId, RoomId, Timestamp},
    signaling::Role,
};
use redis::{FromRedisValue, ToRedisArgs};

use crate::{SignalingModuleError, SignalingRoomId};

#[async_trait::async_trait(?Send)]
pub trait AttributeActions {
    fn set<V: ToRedisArgs>(&mut self, name: &str, value: V) -> &mut Self;
    fn get(&mut self, name: &str) -> &mut Self;
    fn del(&mut self, name: &str) -> &mut Self;

    async fn apply<T: FromRedisValue>(
        &self,
        target: &mut impl ControlStorage<BulkAttributeActions = Self>,
    ) -> Result<T, SignalingModuleError> {
        target.perform_bulk_attribute_actions(self).await
    }
}

#[async_trait(?Send)]
pub trait ControlStorage {
    type BulkAttributeActions: AttributeActions;

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

    fn bulk_attribute_actions(
        &self,
        room: SignalingRoomId,
        participant: ParticipantId,
    ) -> Self::BulkAttributeActions;

    async fn perform_bulk_attribute_actions<T: FromRedisValue>(
        &mut self,
        actions: &Self::BulkAttributeActions,
    ) -> Result<T, SignalingModuleError>;

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

    async fn participants_all_left(
        &mut self,
        room: SignalingRoomId,
    ) -> Result<bool, SignalingModuleError> {
        let participants = self.get_all_participants(room).await?;

        let left_at_attrs: Vec<Option<Timestamp>> = self
            .get_attribute_for_participants(room, "left_at", &Vec::from_iter(participants))
            .await?;

        Ok(left_at_attrs.iter().all(Option::is_some))
    }

    async fn remove_attribute_key(
        &mut self,
        room: SignalingRoomId,
        name: &str,
    ) -> Result<(), SignalingModuleError>;

    async fn get_role_and_left_at_for_room_participants(
        &mut self,
        room: SignalingRoomId,
    ) -> Result<BTreeMap<ParticipantId, (Option<Role>, Option<Timestamp>)>, SignalingModuleError>;

    /// Try to set the active tariff for the room. If the tariff is already set return the current one.
    async fn try_init_tariff(
        &mut self,
        room_id: RoomId,
        tariff: Tariff,
    ) -> Result<Tariff, SignalingModuleError>;

    async fn get_tariff(&mut self, room_id: RoomId) -> Result<Tariff, SignalingModuleError>;

    async fn delete_tariff(&mut self, room_id: RoomId) -> Result<(), SignalingModuleError>;

    /// Try to set the active event for the room. If the event is already set return the current one.
    async fn try_init_event(
        &mut self,
        room_id: RoomId,
        event: Option<Event>,
    ) -> Result<Option<Event>, SignalingModuleError>;

    async fn get_event(&mut self, room_id: RoomId) -> Result<Option<Event>, SignalingModuleError>;

    async fn delete_event(&mut self, room_id: RoomId) -> Result<(), SignalingModuleError>;

    async fn increment_participant_count(
        &mut self,
        room_id: RoomId,
    ) -> Result<isize, SignalingModuleError>;

    async fn decrement_participant_count(
        &mut self,
        room_id: RoomId,
    ) -> Result<isize, SignalingModuleError>;

    async fn get_participant_count(
        &mut self,
        room_id: RoomId,
    ) -> Result<Option<isize>, SignalingModuleError>;

    async fn delete_participant_count(
        &mut self,
        room_id: RoomId,
    ) -> Result<(), SignalingModuleError>;

    async fn set_room_closes_at(
        &mut self,
        room: SignalingRoomId,
        timestamp: Timestamp,
    ) -> Result<(), SignalingModuleError>;

    async fn get_room_closes_at(
        &mut self,
        room: SignalingRoomId,
    ) -> Result<Option<Timestamp>, SignalingModuleError>;

    async fn remove_room_closes_at(
        &mut self,
        room: SignalingRoomId,
    ) -> Result<(), SignalingModuleError>;

    /// Set the `skip_waiting_room` flag for participant with an expiry.
    async fn set_skip_waiting_room_with_expiry(
        &mut self,
        participant: ParticipantId,
        value: bool,
    ) -> Result<(), SignalingModuleError>;

    /// Set the `skip_waiting_room` flag for participant with an expiry if the key does not exist.
    async fn set_skip_waiting_room_with_expiry_nx(
        &mut self,
        participant: ParticipantId,
        value: bool,
    ) -> Result<(), SignalingModuleError>;

    /// Extend the `skip_waiting_room` flag for participant with an expiry.
    async fn reset_skip_waiting_room_expiry(
        &mut self,
        participant: ParticipantId,
    ) -> Result<(), SignalingModuleError>;

    /// Get the `skip_waiting_room` value for participant. If no value is set for the key,
    /// false is returned.
    async fn get_skip_waiting_room(
        &mut self,
        participant: ParticipantId,
    ) -> Result<bool, SignalingModuleError>;
}
