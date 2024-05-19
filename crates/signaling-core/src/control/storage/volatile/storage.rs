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
    signaling::Role,
};
use parking_lot::RwLock;
use redis::FromRedisValue;
use snafu::OptionExt as _;

use super::memory::{MemoryControlState, VolatileStaticMemoryAttributeActions};
use crate::{
    control::storage::ControlStorage, NotFoundSnafu, SignalingModuleError, SignalingRoomId,
    VolatileStaticMemoryStorage,
};

static STATE: OnceLock<Arc<RwLock<MemoryControlState>>> = OnceLock::new();

fn state() -> &'static Arc<RwLock<MemoryControlState>> {
    STATE.get_or_init(Default::default)
}

#[async_trait(?Send)]
impl ControlStorage for VolatileStaticMemoryStorage {
    type BulkAttributeActions = VolatileStaticMemoryAttributeActions;

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

    #[tracing::instrument(level = "debug", skip(self))]
    async fn get_attribute<V>(
        &mut self,
        room: SignalingRoomId,
        participant: ParticipantId,
        name: &str,
    ) -> Result<V, SignalingModuleError>
    where
        V: FromRedisValue,
    {
        state()
            .read()
            .get_attribute(room, participant, name)?
            .with_context(|| NotFoundSnafu)
    }

    #[tracing::instrument(level = "debug", skip(self))]
    async fn set_attribute<V>(
        &mut self,
        room: SignalingRoomId,
        participant: ParticipantId,
        name: &str,
        value: V,
    ) -> Result<(), SignalingModuleError>
    where
        V: core::fmt::Debug + redis::ToRedisArgs + Send + Sync,
    {
        state()
            .write()
            .set_attribute(room, participant, name, value);
        Ok(())
    }

    #[tracing::instrument(level = "debug", skip(self))]
    async fn remove_attribute(
        &mut self,
        room: SignalingRoomId,
        participant: ParticipantId,
        name: &str,
    ) -> Result<(), SignalingModuleError> {
        state().write().remove_attribute(room, participant, name);
        Ok(())
    }

    fn bulk_attribute_actions(
        &self,
        room: SignalingRoomId,
        participant: ParticipantId,
    ) -> Self::BulkAttributeActions {
        VolatileStaticMemoryAttributeActions::new(room, participant)
    }

    #[tracing::instrument(level = "debug", skip(self, actions))]
    async fn perform_bulk_attribute_actions<T: FromRedisValue>(
        &mut self,
        actions: &Self::BulkAttributeActions,
    ) -> Result<T, SignalingModuleError> {
        state().write().perform_bulk_attribute_actions(actions)
    }

    #[tracing::instrument(level = "debug", skip(self))]
    async fn get_attribute_for_participants<V>(
        &mut self,
        room: SignalingRoomId,
        name: &str,
        participants: &[ParticipantId],
    ) -> Result<Vec<Option<V>>, SignalingModuleError>
    where
        V: FromRedisValue,
    {
        state()
            .read()
            .get_attribute_for_participants(room, name, participants)
    }

    #[tracing::instrument(level = "debug", skip(self))]
    async fn remove_attribute_key(
        &mut self,
        room: SignalingRoomId,
        name: &str,
    ) -> Result<(), SignalingModuleError> {
        state().write().remove_attribute_key(room, name);
        Ok(())
    }

    #[tracing::instrument(level = "debug", skip(self))]
    async fn get_role_and_left_at_for_room_participants(
        &mut self,
        room: SignalingRoomId,
    ) -> Result<BTreeMap<ParticipantId, (Option<Role>, Option<Timestamp>)>, SignalingModuleError>
    {
        state()
            .read()
            .get_role_and_left_at_for_room_participants(room)
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
    async fn room_closes_at() {
        test_common::room_closes_at(&mut storage().await).await;
    }
}
