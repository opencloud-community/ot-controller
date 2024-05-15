// SPDX-FileCopyrightText: OpenTalk GmbH <mail@opentalk.eu>
//
// SPDX-License-Identifier: EUPL-1.2

use std::{
    collections::BTreeSet,
    sync::{Arc, OnceLock},
};

use async_trait::async_trait;
use opentalk_types::core::ParticipantId;
use parking_lot::RwLock;
use redis::FromRedisValue;

use super::memory::MemoryControlState;
use crate::{
    control::storage::control_storage::ControlStorage, SignalingModuleError, SignalingRoomId,
    VolatileStaticMemoryStorage,
};

static STATE: OnceLock<Arc<RwLock<MemoryControlState>>> = OnceLock::new();

fn state() -> &'static Arc<RwLock<MemoryControlState>> {
    STATE.get_or_init(Default::default)
}

#[async_trait(?Send)]
impl ControlStorage for VolatileStaticMemoryStorage {
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
        state().read().get_attribute(room, participant, name)
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
            .set_attribute(room, participant, name, value)
    }

    #[tracing::instrument(level = "debug", skip(self))]
    async fn remove_attribute(
        &mut self,
        room: SignalingRoomId,
        participant: ParticipantId,
        name: &str,
    ) -> Result<(), SignalingModuleError> {
        state().write().remove_attribute(room, participant, name)
    }
}

#[cfg(test)]
mod tests {
    use serial_test::serial;

    use super::super::super::test_common;
    use crate::VolatileStaticMemoryStorage;

    #[tokio::test]
    #[serial]
    async fn participant_set() {
        test_common::participant_set(&mut VolatileStaticMemoryStorage).await;
    }

    #[tokio::test]
    #[serial]
    async fn participant_attribute() {
        test_common::participant_attribute(&mut VolatileStaticMemoryStorage).await;
    }
}
