// SPDX-FileCopyrightText: OpenTalk GmbH <mail@opentalk.eu>
//
// SPDX-License-Identifier: EUPL-1.2

use std::{
    collections::{BTreeMap, BTreeSet},
    sync::{Arc, OnceLock},
};

use async_trait::async_trait;
use opentalk_db_storage::{events::Event, tariffs::Tariff};
use opentalk_types::{
    core::{ParticipantId, RoomId, Timestamp},
    signaling::{control::room::CreatorInfo, Role},
};
use parking_lot::RwLock;

use super::memory::MemoryControlState;
use crate::{
    control::storage::{
        control_storage::{ControlStorageParticipantSet, ControlStorageSkipWaitingRoom},
        AttributeActions, AttributeId, ControlStorage, ControlStorageEvent,
        ControlStorageParticipantAttributes, ControlStorageParticipantAttributesRaw, LEFT_AT, ROLE,
    },
    SignalingModuleError, SignalingRoomId, VolatileStaticMemoryStorage,
};

static STATE: OnceLock<Arc<RwLock<MemoryControlState>>> = OnceLock::new();

fn state() -> &'static Arc<RwLock<MemoryControlState>> {
    STATE.get_or_init(Default::default)
}

#[async_trait(?Send)]
impl ControlStorage for VolatileStaticMemoryStorage {
    #[tracing::instrument(level = "debug", skip(self))]
    async fn remove_attribute_key(
        &mut self,
        room: SignalingRoomId,
        attribute: AttributeId,
    ) -> Result<(), SignalingModuleError> {
        state().write().remove_attribute_key(room, attribute);
        Ok(())
    }

    #[tracing::instrument(level = "debug", skip(self))]
    async fn get_role_and_left_at_for_room_participants(
        &mut self,
        room: SignalingRoomId,
    ) -> Result<BTreeMap<ParticipantId, (Option<Role>, Option<Timestamp>)>, SignalingModuleError>
    {
        let participants = Vec::from_iter(self.get_all_participants(room).await?);

        let roles = self
            .get_attribute_for_participants(room, &participants, ROLE)
            .await?;
        let left_at = self
            .get_attribute_for_participants(room, &participants, LEFT_AT)
            .await?;

        Ok(participants
            .into_iter()
            .zip(std::iter::zip(roles, left_at))
            .collect())
    }

    #[tracing::instrument(level = "debug", skip(self))]
    async fn try_init_tariff(
        &mut self,
        room_id: RoomId,
        tariff: Tariff,
    ) -> Result<Tariff, SignalingModuleError> {
        Ok(state().write().try_init_tariff(room_id, tariff))
    }

    #[tracing::instrument(level = "debug", skip(self))]
    async fn get_tariff(&mut self, room_id: RoomId) -> Result<Tariff, SignalingModuleError> {
        state().write().get_tariff(room_id)
    }

    #[tracing::instrument(level = "debug", skip(self))]
    async fn delete_tariff(&mut self, room_id: RoomId) -> Result<(), SignalingModuleError> {
        state().write().delete_tariff(room_id);
        Ok(())
    }

    #[tracing::instrument(level = "debug", skip(self))]
    async fn increment_participant_count(
        &mut self,
        room_id: RoomId,
    ) -> Result<isize, SignalingModuleError> {
        Ok(state().write().increment_participant_count(room_id))
    }

    #[tracing::instrument(level = "debug", skip(self))]
    async fn decrement_participant_count(
        &mut self,
        room_id: RoomId,
    ) -> Result<isize, SignalingModuleError> {
        Ok(state().write().decrement_participant_count(room_id))
    }

    #[tracing::instrument(level = "debug", skip(self))]
    async fn get_participant_count(
        &mut self,
        room_id: RoomId,
    ) -> Result<Option<isize>, SignalingModuleError> {
        Ok(state().read().get_participant_count(room_id))
    }

    #[tracing::instrument(level = "debug", skip(self))]
    async fn delete_participant_count(
        &mut self,
        room_id: RoomId,
    ) -> Result<(), SignalingModuleError> {
        state().write().delete_participant_count(room_id);
        Ok(())
    }

    async fn try_init_creator(
        &mut self,
        room_id: RoomId,
        creator: CreatorInfo,
    ) -> Result<CreatorInfo, SignalingModuleError> {
        Ok(state().write().try_init_creator(room_id, creator))
    }

    async fn get_creator(
        &mut self,
        room_id: RoomId,
    ) -> Result<Option<CreatorInfo>, SignalingModuleError> {
        Ok(state().read().get_creator(room_id))
    }

    async fn delete_creator(&mut self, room_id: RoomId) -> Result<(), SignalingModuleError> {
        state().write().delete_creator(room_id);
        Ok(())
    }

    #[tracing::instrument(level = "debug", skip(self))]
    async fn set_room_closes_at(
        &mut self,
        room: SignalingRoomId,
        timestamp: Timestamp,
    ) -> Result<(), SignalingModuleError> {
        state().write().set_room_closes_at(room, timestamp);
        Ok(())
    }

    #[tracing::instrument(level = "debug", skip(self))]
    async fn get_room_closes_at(
        &mut self,
        room: SignalingRoomId,
    ) -> Result<Option<Timestamp>, SignalingModuleError> {
        Ok(state().read().get_room_closes_at(room))
    }

    #[tracing::instrument(level = "debug", skip(self))]
    async fn remove_room_closes_at(
        &mut self,
        room: SignalingRoomId,
    ) -> Result<(), SignalingModuleError> {
        state().write().remove_room_closes_at(room);
        Ok(())
    }
}

#[async_trait(?Send)]
impl ControlStorageSkipWaitingRoom for VolatileStaticMemoryStorage {
    #[tracing::instrument(level = "debug", skip(self))]
    async fn set_skip_waiting_room_with_expiry(
        &mut self,
        participant: ParticipantId,
        value: bool,
    ) -> Result<(), SignalingModuleError> {
        state()
            .write()
            .set_skip_waiting_room_with_expiry(participant, value);
        Ok(())
    }

    #[tracing::instrument(level = "debug", skip(self))]
    async fn set_skip_waiting_room_with_expiry_nx(
        &mut self,
        participant: ParticipantId,
        value: bool,
    ) -> Result<(), SignalingModuleError> {
        state()
            .write()
            .set_skip_waiting_room_with_expiry_nx(participant, value);
        Ok(())
    }

    #[tracing::instrument(level = "debug", skip(self))]
    async fn reset_skip_waiting_room_expiry(
        &mut self,
        participant: ParticipantId,
    ) -> Result<(), SignalingModuleError> {
        state().write().reset_skip_waiting_room_expiry(participant);
        Ok(())
    }

    #[tracing::instrument(level = "debug", skip(self))]
    async fn get_skip_waiting_room(
        &mut self,
        participant: ParticipantId,
    ) -> Result<bool, SignalingModuleError> {
        Ok(state().read().get_skip_waiting_room(participant))
    }
}

#[async_trait(?Send)]
impl ControlStorageParticipantSet for VolatileStaticMemoryStorage {
    #[tracing::instrument(level = "debug", skip(self))]
    async fn participant_set_exists(
        &mut self,
        room: SignalingRoomId,
    ) -> Result<bool, SignalingModuleError> {
        Ok(state().read().participant_set_exists(room))
    }

    #[tracing::instrument(level = "debug", skip(self))]
    async fn get_all_participants(
        &mut self,
        room: SignalingRoomId,
    ) -> Result<BTreeSet<ParticipantId>, SignalingModuleError> {
        Ok(state().read().get_all_participants(room))
    }

    #[tracing::instrument(level = "debug", skip(self))]
    async fn remove_participant_set(
        &mut self,
        room: SignalingRoomId,
    ) -> Result<(), SignalingModuleError> {
        state().write().remove_participant_set(room);
        Ok(())
    }

    #[tracing::instrument(level = "debug", skip(self))]
    async fn participants_contains(
        &mut self,
        room: SignalingRoomId,
        participant: ParticipantId,
    ) -> Result<bool, SignalingModuleError> {
        Ok(state().read().participants_contains(room, participant))
    }

    #[tracing::instrument(level = "debug", skip(self))]
    async fn check_participants_exist(
        &mut self,
        room: SignalingRoomId,
        participants: &[ParticipantId],
    ) -> Result<bool, SignalingModuleError> {
        Ok(state().read().check_participants_exist(room, participants))
    }

    #[tracing::instrument(level = "debug", skip(self))]
    async fn add_participant_to_set(
        &mut self,
        room: SignalingRoomId,
        participant: ParticipantId,
    ) -> Result<bool, SignalingModuleError> {
        Ok(state().write().add_participant_to_set(room, participant))
    }
}

#[async_trait(?Send)]
impl ControlStorageParticipantAttributesRaw for VolatileStaticMemoryStorage {
    #[tracing::instrument(level = "debug", skip(self))]
    async fn get_attribute_raw(
        &mut self,
        room: SignalingRoomId,
        participant: ParticipantId,
        attribute: AttributeId,
    ) -> Result<serde_json::Value, SignalingModuleError> {
        Ok(state()
            .read()
            .get_attribute_raw(room, participant, attribute))
    }

    #[tracing::instrument(level = "debug", skip(self))]
    async fn get_attribute_for_participants_raw(
        &mut self,
        room: SignalingRoomId,
        participants: &[ParticipantId],
        attribute: AttributeId,
    ) -> Result<Vec<serde_json::Value>, SignalingModuleError> {
        Ok(state()
            .read()
            .get_attribute_for_participants_raw(room, participants, attribute))
    }

    #[tracing::instrument(level = "debug", skip(self))]
    async fn set_attribute_raw(
        &mut self,
        room: SignalingRoomId,
        participant: ParticipantId,
        attribute: AttributeId,
        value: serde_json::Value,
    ) -> Result<(), SignalingModuleError> {
        state()
            .write()
            .set_attribute_raw(room, participant, attribute, value);
        Ok(())
    }

    #[tracing::instrument(level = "debug", skip(self))]
    async fn remove_attribute_raw(
        &mut self,
        room: SignalingRoomId,
        participant: ParticipantId,
        attribute: AttributeId,
    ) -> Result<(), SignalingModuleError> {
        state()
            .write()
            .remove_attribute_raw(room, participant, attribute);
        Ok(())
    }

    #[tracing::instrument(level = "debug", skip(self, actions))]
    async fn bulk_attribute_actions_raw(
        &mut self,
        actions: &AttributeActions,
    ) -> Result<serde_json::Value, SignalingModuleError> {
        state().write().perform_bulk_attribute_actions_raw(actions)
    }
}

#[async_trait(?Send)]
impl ControlStorageEvent for VolatileStaticMemoryStorage {
    #[tracing::instrument(level = "debug", skip(self))]
    async fn try_init_event(
        &mut self,
        room_id: RoomId,
        event: Option<Event>,
    ) -> Result<Option<Event>, SignalingModuleError> {
        Ok(state().write().try_init_event(room_id, event))
    }

    #[tracing::instrument(level = "debug", skip(self))]
    async fn get_event(&mut self, room_id: RoomId) -> Result<Option<Event>, SignalingModuleError> {
        state().read().get_event(room_id)
    }

    #[tracing::instrument(level = "debug", skip(self))]
    async fn delete_event(&mut self, room_id: RoomId) -> Result<(), SignalingModuleError> {
        state().write().delete_event(room_id);
        Ok(())
    }
}

#[cfg(test)]
mod tests {
    use serial_test::serial;

    use super::{super::super::test_common, state};
    use crate::VolatileStaticMemoryStorage;

    async fn storage() -> VolatileStaticMemoryStorage {
        state().write().reset();
        VolatileStaticMemoryStorage
    }

    #[tokio::test]
    #[serial]
    async fn participant_set() {
        test_common::participant_set(&mut storage().await).await;
    }

    #[tokio::test]
    #[serial]
    async fn participant_attribute() {
        test_common::participant_attribute(&mut storage().await).await;
    }

    #[tokio::test]
    #[serial]
    async fn participant_attributes() {
        test_common::participant_attributes(&mut storage().await).await;
    }

    #[tokio::test]
    #[serial]
    async fn participant_remove_attributes() {
        test_common::participant_remove_attributes(&mut storage().await).await;
    }

    #[tokio::test]
    #[serial]
    async fn get_role_and_left_for_room_participants() {
        test_common::get_role_and_left_for_room_participants(&mut storage().await).await;
    }

    #[tokio::test]
    #[serial]
    async fn participant_attributes_bulk() {
        test_common::participant_attributes_bulk(&mut storage().await).await;
    }

    #[tokio::test]
    #[serial]
    async fn tariff() {
        test_common::tariff(&mut storage().await).await;
    }

    #[tokio::test]
    #[serial]
    async fn event() {
        test_common::event(&mut storage().await).await;
    }

    #[tokio::test]
    #[serial]
    async fn participant_count() {
        test_common::participant_count(&mut storage().await).await;
    }

    #[tokio::test]
    #[serial]
    async fn creator_info() {
        test_common::creator_info(&mut storage().await).await;
    }

    #[tokio::test]
    #[serial]
    async fn room_closes_at() {
        test_common::room_closes_at(&mut storage().await).await;
    }

    #[tokio::test]
    #[serial]
    async fn skip_waiting_room() {
        test_common::skip_waiting_room(&mut storage().await).await;
    }
}
