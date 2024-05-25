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
use serde::{de::DeserializeOwned, Serialize};
use snafu::ResultExt as _;

use super::LEFT_AT;
use crate::{SerdeJsonSnafu, SignalingModuleError, SignalingRoomId};

#[derive(
    Debug,
    Clone,
    Copy,
    PartialEq,
    Eq,
    PartialOrd,
    Ord,
    Hash,
    derive_more::Display,
    derive_more::From,
    derive_more::Into,
)]
pub struct AttributeId(&'static str);

impl AttributeId {
    pub const fn new(identifier: &'static str) -> Self {
        Self(identifier)
    }
}

#[async_trait::async_trait(?Send)]
pub trait AttributeActions {
    fn set<V: Serialize>(&mut self, attribute: AttributeId, value: V) -> &mut Self;
    fn get(&mut self, attribute: AttributeId) -> &mut Self;
    fn del(&mut self, attribute: AttributeId) -> &mut Self;

    async fn apply<T: DeserializeOwned>(
        &self,
        target: &mut impl ControlStorage<BulkAttributeActions = Self>,
    ) -> Result<T, SignalingModuleError> {
        target.perform_bulk_attribute_actions(self).await
    }
}

#[async_trait(?Send)]
pub trait ControlStorage:
    ControlStorageParticipantAttributes + ControlStorageParticipantAttributesBulk
{
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

    async fn participants_all_left(
        &mut self,
        room: SignalingRoomId,
    ) -> Result<bool, SignalingModuleError> {
        let participants = self.get_all_participants(room).await?;

        let left_at_attrs: Vec<Option<Timestamp>> = self
            .get_attribute_for_participants(room, &Vec::from_iter(participants), LEFT_AT)
            .await?;

        Ok(left_at_attrs.iter().all(Option::is_some))
    }

    async fn remove_attribute_key(
        &mut self,
        room: SignalingRoomId,
        attribute: AttributeId,
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

#[async_trait(?Send)]
pub trait ControlStorageParticipantAttributesBulk {
    type BulkAttributeActions: AttributeActions;

    fn bulk_attribute_actions(
        &self,
        room: SignalingRoomId,
        participant: ParticipantId,
    ) -> Self::BulkAttributeActions;

    async fn perform_bulk_attribute_actions<T: DeserializeOwned>(
        &mut self,
        actions: &Self::BulkAttributeActions,
    ) -> Result<T, SignalingModuleError>;
}

#[async_trait(?Send)]
pub trait ControlStorageParticipantAttributes: ControlStorageParticipantAttributesRaw {
    async fn get_attribute<V>(
        &mut self,
        room: SignalingRoomId,
        participant: ParticipantId,
        attribute: AttributeId,
    ) -> Result<V, SignalingModuleError>
    where
        V: DeserializeOwned,
    {
        let loaded = self.get_attribute_raw(room, participant, attribute).await?;
        let deserialized = serde_json::from_value(loaded).with_context(|e| SerdeJsonSnafu{
                message: format!("failed to deserialize attribute {attribute} for participant {participant} in room {room}, {e}")
        })?;
        Ok(deserialized)
    }

    async fn get_attribute_for_participants<V>(
        &mut self,
        room: SignalingRoomId,
        participants: &[ParticipantId],
        attribute: AttributeId,
    ) -> Result<Vec<Option<V>>, SignalingModuleError>
    where
        V: DeserializeOwned,
    {
        let loaded = self
            .get_attribute_for_participants_raw(room, participants, attribute)
            .await?;

        loaded.into_iter().map(|v|

        serde_json::from_value(v).with_context(|e| SerdeJsonSnafu{
                message: format!("failed to deserialize attribute {attribute} multiple for participants {participants:?} in room {room}, {e}")
        })).collect()
    }

    async fn set_attribute<V>(
        &mut self,
        room: SignalingRoomId,
        participant: ParticipantId,
        attribute: AttributeId,
        value: V,
    ) -> Result<(), SignalingModuleError>
    where
        V: core::fmt::Debug + Serialize + Send + Sync,
    {
        let serialized = serde_json::to_value(value).with_context(|e| SerdeJsonSnafu {
            message: format!("failed to serialize attribute {attribute} for participant {participant} in room {room}, {e}")
        })?;
        self.set_attribute_raw(room, participant, attribute, serialized)
            .await
    }

    async fn remove_attribute(
        &mut self,
        room: SignalingRoomId,
        participant: ParticipantId,
        attribute: AttributeId,
    ) -> Result<(), SignalingModuleError> {
        self.remove_attribute_raw(room, participant, attribute)
            .await
    }
}

impl<T: ControlStorageParticipantAttributesRaw> ControlStorageParticipantAttributes for T {}

#[async_trait(?Send)]
pub trait ControlStorageParticipantAttributesRaw {
    async fn get_attribute_raw(
        &mut self,
        room: SignalingRoomId,
        participant: ParticipantId,
        attribute: AttributeId,
    ) -> Result<serde_json::Value, SignalingModuleError>;

    async fn get_attribute_for_participants_raw(
        &mut self,
        room: SignalingRoomId,
        participants: &[ParticipantId],
        attribute: AttributeId,
    ) -> Result<Vec<serde_json::Value>, SignalingModuleError>;

    async fn set_attribute_raw(
        &mut self,
        room: SignalingRoomId,
        participant: ParticipantId,
        attribute: AttributeId,
        value: serde_json::Value,
    ) -> Result<(), SignalingModuleError>;

    async fn remove_attribute_raw(
        &mut self,
        room: SignalingRoomId,
        participant: ParticipantId,
        attribute: AttributeId,
    ) -> Result<(), SignalingModuleError>;
}
