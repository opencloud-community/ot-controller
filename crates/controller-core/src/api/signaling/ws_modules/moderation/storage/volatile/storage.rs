// SPDX-FileCopyrightText: OpenTalk GmbH <mail@opentalk.eu>
//
// SPDX-License-Identifier: EUPL-1.2

use std::{
    collections::BTreeSet,
    sync::{Arc, OnceLock},
};

use async_trait::async_trait;
use opentalk_signaling_core::{SignalingModuleError, VolatileStaticMemoryStorage};
use opentalk_types::core::{RoomId, UserId};
use opentalk_types_signaling::ParticipantId;
use parking_lot::RwLock;

use super::memory::MemoryModerationState;
use crate::api::signaling::moderation::storage::ModerationStorage;

static STATE: OnceLock<Arc<RwLock<MemoryModerationState>>> = OnceLock::new();

fn state() -> &'static Arc<RwLock<MemoryModerationState>> {
    STATE.get_or_init(Default::default)
}

#[async_trait(?Send)]
impl ModerationStorage for VolatileStaticMemoryStorage {
    #[tracing::instrument(level = "debug", skip(self))]
    async fn ban_user(&mut self, room: RoomId, user: UserId) -> Result<(), SignalingModuleError> {
        state().write().ban_user(room, user);
        Ok(())
    }

    #[tracing::instrument(level = "debug", skip(self))]
    async fn is_user_banned(
        &mut self,
        room: RoomId,
        user: UserId,
    ) -> Result<bool, SignalingModuleError> {
        Ok(state().read().is_user_banned(room, user))
    }

    #[tracing::instrument(level = "debug", skip(self))]
    async fn delete_user_bans(&mut self, room: RoomId) -> Result<(), SignalingModuleError> {
        state().write().delete_user_bans(room);
        Ok(())
    }

    #[tracing::instrument(level = "debug", skip(self))]
    async fn init_waiting_room_enabled(
        &mut self,
        room: RoomId,
        enabled: bool,
    ) -> Result<bool, SignalingModuleError> {
        Ok(state().write().init_waiting_room_enabled(room, enabled))
    }

    #[tracing::instrument(level = "debug", skip(self))]
    async fn set_waiting_room_enabled(
        &mut self,
        room: RoomId,
        enabled: bool,
    ) -> Result<(), SignalingModuleError> {
        state().write().set_waiting_room_enabled(room, enabled);
        Ok(())
    }

    #[tracing::instrument(level = "debug", skip(self))]
    async fn is_waiting_room_enabled(
        &mut self,
        room: RoomId,
    ) -> Result<bool, SignalingModuleError> {
        Ok(state().read().is_waiting_room_enabled(room))
    }

    #[tracing::instrument(level = "debug", skip(self))]
    async fn delete_waiting_room_enabled(
        &mut self,
        room: RoomId,
    ) -> Result<(), SignalingModuleError> {
        state().write().delete_waiting_room_enabled(room);
        Ok(())
    }

    #[tracing::instrument(level = "debug", skip(self))]
    async fn set_raise_hands_enabled(
        &mut self,
        room: RoomId,
        enabled: bool,
    ) -> Result<(), SignalingModuleError> {
        state().write().set_raise_hands_enabled(room, enabled);
        Ok(())
    }

    #[tracing::instrument(level = "debug", skip(self))]
    async fn is_raise_hands_enabled(&mut self, room: RoomId) -> Result<bool, SignalingModuleError> {
        Ok(state().read().is_raise_hands_enabled(room))
    }

    #[tracing::instrument(level = "debug", skip(self))]
    async fn delete_raise_hands_enabled(
        &mut self,
        room: RoomId,
    ) -> Result<(), SignalingModuleError> {
        state().write().delete_raise_hands_enabled(room);
        Ok(())
    }

    #[tracing::instrument(level = "debug", skip(self))]
    async fn waiting_room_add_participant(
        &mut self,
        room: RoomId,
        participant: ParticipantId,
    ) -> Result<bool, SignalingModuleError> {
        Ok(state()
            .write()
            .waiting_room_add_participant(room, participant))
    }

    #[tracing::instrument(level = "debug", skip(self))]
    async fn waiting_room_remove_participant(
        &mut self,
        room: RoomId,
        participant: ParticipantId,
    ) -> Result<(), SignalingModuleError> {
        state()
            .write()
            .waiting_room_remove_participant(room, participant);
        Ok(())
    }

    #[tracing::instrument(level = "debug", skip(self))]
    async fn waiting_room_contains_participant(
        &mut self,
        room: RoomId,
        participant: ParticipantId,
    ) -> Result<bool, SignalingModuleError> {
        Ok(state()
            .read()
            .waiting_room_contains_participant(room, participant))
    }

    #[tracing::instrument(level = "debug", skip(self))]
    async fn waiting_room_participants(
        &mut self,
        room: RoomId,
    ) -> Result<BTreeSet<ParticipantId>, SignalingModuleError> {
        Ok(state().read().waiting_room_participants(room))
    }

    #[tracing::instrument(level = "debug", skip(self))]
    async fn waiting_room_participant_count(
        &mut self,
        room: RoomId,
    ) -> Result<usize, SignalingModuleError> {
        Ok(state().read().waiting_room_participant_count(room))
    }

    #[tracing::instrument(level = "debug", skip(self))]
    async fn delete_waiting_room(&mut self, room: RoomId) -> Result<(), SignalingModuleError> {
        state().write().delete_waiting_room(room);
        Ok(())
    }

    #[tracing::instrument(level = "debug", skip(self))]
    async fn waiting_room_accepted_add_participant(
        &mut self,
        room: RoomId,
        participant: ParticipantId,
    ) -> Result<bool, SignalingModuleError> {
        Ok(state()
            .write()
            .waiting_room_accepted_add_participant(room, participant))
    }

    #[tracing::instrument(level = "debug", skip(self))]
    async fn waiting_room_accepted_remove_participant(
        &mut self,
        room: RoomId,
        participant: ParticipantId,
    ) -> Result<(), SignalingModuleError> {
        state()
            .write()
            .waiting_room_accepted_remove_participant(room, participant);
        Ok(())
    }

    #[tracing::instrument(level = "debug", skip(self))]
    async fn waiting_room_accepted_remove_participants(
        &mut self,
        room: RoomId,
        participants: &[ParticipantId],
    ) -> Result<(), SignalingModuleError> {
        state()
            .write()
            .waiting_room_accepted_remove_participants(room, participants);
        Ok(())
    }

    #[tracing::instrument(level = "debug", skip(self))]
    async fn waiting_room_accepted_participants(
        &mut self,
        room: RoomId,
    ) -> Result<BTreeSet<ParticipantId>, SignalingModuleError> {
        Ok(state().read().waiting_room_accepted_participants(room))
    }

    #[tracing::instrument(level = "debug", skip(self))]
    async fn waiting_room_accepted_participant_count(
        &mut self,
        room: RoomId,
    ) -> Result<usize, SignalingModuleError> {
        Ok(state().read().waiting_room_accepted_participant_count(room))
    }

    #[tracing::instrument(level = "debug", skip(self))]
    async fn delete_waiting_room_accepted(
        &mut self,
        room: RoomId,
    ) -> Result<(), SignalingModuleError> {
        state().write().delete_waiting_room_accepted(room);
        Ok(())
    }
}

#[cfg(test)]
mod test {
    use opentalk_signaling_core::VolatileStaticMemoryStorage;
    use serial_test::serial;

    use super::{super::super::test_common, state};

    async fn storage() -> VolatileStaticMemoryStorage {
        state().write().reset();
        VolatileStaticMemoryStorage
    }

    #[tokio::test]
    #[serial]
    async fn user_bans() {
        test_common::user_bans(&mut storage().await).await;
    }

    #[tokio::test]
    #[serial]
    async fn waiting_room_enabled_flag() {
        test_common::waiting_room_enabled_flag(&mut storage().await).await;
    }

    #[tokio::test]
    #[serial]
    async fn raise_hands_enabled_flag() {
        test_common::raise_hands_enabled_flag(&mut storage().await).await;
    }

    #[tokio::test]
    #[serial]
    async fn waiting_room_participants() {
        test_common::waiting_room_participants(&mut storage().await).await;
    }

    #[tokio::test]
    #[serial]
    async fn waiting_room_accepted_participants() {
        test_common::waiting_room_accepted_participants(&mut storage().await).await;
    }
}
