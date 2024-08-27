// SPDX-FileCopyrightText: OpenTalk GmbH <mail@opentalk.eu>
//
// SPDX-License-Identifier: EUPL-1.2

use std::{
    collections::{HashMap, HashSet},
    sync::{Arc, OnceLock},
};

use async_trait::async_trait;
use opentalk_signaling_core::{SignalingModuleError, SignalingRoomId, VolatileStaticMemoryStorage};
use opentalk_types::{
    core::{ParticipantId, RoomId, Timestamp},
    signaling::chat::state::StoredMessage,
};
use opentalk_types_common::users::{GroupId, GroupName};
use parking_lot::RwLock;

use super::memory::MemoryChatState;
use crate::{storage::chat_storage::ChatStorage, ParticipantPair};

static STATE: OnceLock<Arc<RwLock<MemoryChatState>>> = OnceLock::new();

fn state() -> &'static Arc<RwLock<MemoryChatState>> {
    STATE.get_or_init(Default::default)
}

#[async_trait(?Send)]
impl ChatStorage for VolatileStaticMemoryStorage {
    #[tracing::instrument(level = "debug", skip(self))]
    async fn get_room_history(
        &mut self,
        room: SignalingRoomId,
    ) -> Result<Vec<StoredMessage>, SignalingModuleError> {
        Ok(state().read().get_room_history(room))
    }

    #[tracing::instrument(level = "debug", skip(self))]
    async fn add_message_to_room_history(
        &mut self,
        room: SignalingRoomId,
        message: &StoredMessage,
    ) -> Result<(), SignalingModuleError> {
        state().write().add_message_to_room_history(room, message);
        Ok(())
    }

    #[tracing::instrument(level = "debug", skip(self))]
    async fn delete_room_history(
        &mut self,
        room: SignalingRoomId,
    ) -> Result<(), SignalingModuleError> {
        state().write().delete_room_history(room);
        Ok(())
    }

    #[tracing::instrument(level = "debug", skip(self))]
    async fn set_chat_enabled(
        &mut self,
        room: RoomId,
        enabled: bool,
    ) -> Result<(), SignalingModuleError> {
        state().write().set_chat_enabled(room, enabled);
        Ok(())
    }

    #[tracing::instrument(level = "debug", skip(self))]
    async fn is_chat_enabled(&mut self, room: RoomId) -> Result<bool, SignalingModuleError> {
        Ok(state().read().is_chat_enabled(room))
    }

    #[tracing::instrument(level = "debug", skip(self))]
    async fn delete_chat_enabled(&mut self, room: RoomId) -> Result<(), SignalingModuleError> {
        state().write().delete_chat_enabled(room);
        Ok(())
    }

    #[tracing::instrument(level = "debug", skip(self))]
    async fn set_last_seen_timestamps_private(
        &mut self,
        room: SignalingRoomId,
        participant: ParticipantId,
        timestamps: &[(ParticipantId, Timestamp)],
    ) -> Result<(), SignalingModuleError> {
        state()
            .write()
            .set_last_seen_timestamps_private(room, participant, timestamps);
        Ok(())
    }

    #[tracing::instrument(level = "debug", skip(self))]
    async fn get_last_seen_timestamps_private(
        &mut self,
        room: SignalingRoomId,
        participant: ParticipantId,
    ) -> Result<HashMap<ParticipantId, Timestamp>, SignalingModuleError> {
        Ok(state()
            .read()
            .get_last_seen_timestamps_private(room, participant))
    }

    #[tracing::instrument(level = "debug", skip(self))]
    async fn delete_last_seen_timestamps_private(
        &mut self,
        room: SignalingRoomId,
        participant: ParticipantId,
    ) -> Result<(), SignalingModuleError> {
        state()
            .write()
            .delete_last_seen_timestamps_private(room, participant);
        Ok(())
    }

    #[tracing::instrument(level = "debug", skip(self))]
    async fn set_last_seen_timestamps_group(
        &mut self,
        room: SignalingRoomId,
        participant: ParticipantId,
        timestamps: &[(GroupName, Timestamp)],
    ) -> Result<(), SignalingModuleError> {
        state()
            .write()
            .set_last_seen_timestamps_group(room, participant, timestamps);
        Ok(())
    }

    #[tracing::instrument(level = "debug", skip(self))]
    async fn get_last_seen_timestamps_group(
        &mut self,
        room: SignalingRoomId,
        participant: ParticipantId,
    ) -> Result<HashMap<GroupName, Timestamp>, SignalingModuleError> {
        Ok(state()
            .read()
            .get_last_seen_timestamps_group(room, participant))
    }

    #[tracing::instrument(level = "debug", skip(self))]
    async fn delete_last_seen_timestamps_group(
        &mut self,
        room: SignalingRoomId,
        participant: ParticipantId,
    ) -> Result<(), SignalingModuleError> {
        state()
            .write()
            .delete_last_seen_timestamps_group(room, participant);
        Ok(())
    }

    #[tracing::instrument(level = "debug", skip(self))]
    async fn set_last_seen_timestamp_global(
        &mut self,
        room: SignalingRoomId,
        participant: ParticipantId,
        timestamp: Timestamp,
    ) -> Result<(), SignalingModuleError> {
        state()
            .write()
            .set_last_seen_timestamp_global(room, participant, timestamp);
        Ok(())
    }

    #[tracing::instrument(level = "debug", skip(self))]
    async fn get_last_seen_timestamp_global(
        &mut self,
        room: SignalingRoomId,
        participant: ParticipantId,
    ) -> Result<Option<Timestamp>, SignalingModuleError> {
        Ok(state()
            .read()
            .get_last_seen_timestamp_global(room, participant))
    }

    #[tracing::instrument(level = "debug", skip(self))]
    async fn delete_last_seen_timestamp_global(
        &mut self,
        room: SignalingRoomId,
        participant: ParticipantId,
    ) -> Result<(), SignalingModuleError> {
        state()
            .write()
            .delete_last_seen_timestamp_global(room, participant);
        Ok(())
    }

    #[tracing::instrument(level = "debug", skip(self))]
    async fn add_private_chat_correspondents(
        &mut self,
        room: SignalingRoomId,
        participant_one: ParticipantId,
        participant_two: ParticipantId,
    ) -> Result<(), SignalingModuleError> {
        state()
            .write()
            .add_private_chat_correspondents(room, participant_one, participant_two);
        Ok(())
    }

    #[tracing::instrument(level = "debug", skip(self))]
    async fn delete_private_chat_correspondents(
        &mut self,
        room: SignalingRoomId,
    ) -> Result<(), SignalingModuleError> {
        state().write().delete_private_chat_correspondents(room);
        Ok(())
    }

    #[tracing::instrument(level = "debug", skip(self))]
    async fn get_private_chat_correspondents(
        &mut self,
        room: SignalingRoomId,
    ) -> Result<HashSet<ParticipantPair>, SignalingModuleError> {
        Ok(state().read().get_private_chat_correspondents(room))
    }

    #[tracing::instrument(level = "debug", skip(self))]
    async fn get_group_chat_history(
        &mut self,
        room: SignalingRoomId,
        group: GroupId,
    ) -> Result<Vec<StoredMessage>, SignalingModuleError> {
        Ok(state().read().get_group_chat_history(room, group))
    }

    #[tracing::instrument(level = "debug", skip(self, message))]
    async fn add_message_to_group_chat_history(
        &mut self,
        room: SignalingRoomId,
        group: GroupId,
        message: &StoredMessage,
    ) -> Result<(), SignalingModuleError> {
        state()
            .write()
            .add_message_to_group_chat_history(room, group, message);
        Ok(())
    }

    #[tracing::instrument(level = "debug", skip(self))]
    async fn delete_group_chat_history(
        &mut self,
        room: SignalingRoomId,
        group: GroupId,
    ) -> Result<(), SignalingModuleError> {
        state().write().delete_group_chat_history(room, group);
        Ok(())
    }

    #[tracing::instrument(level = "debug", skip(self))]
    async fn get_private_chat_history(
        &mut self,
        room: SignalingRoomId,
        participant_one: ParticipantId,
        participant_two: ParticipantId,
    ) -> Result<Vec<StoredMessage>, SignalingModuleError> {
        Ok(state()
            .read()
            .get_private_chat_history(room, participant_one, participant_two))
    }

    #[tracing::instrument(level = "debug", skip(self, message))]
    async fn add_message_to_private_chat_history(
        &mut self,
        room: SignalingRoomId,
        participant_one: ParticipantId,
        participant_two: ParticipantId,
        message: &StoredMessage,
    ) -> Result<(), SignalingModuleError> {
        state().write().add_message_to_private_chat_history(
            room,
            participant_one,
            participant_two,
            message,
        );
        Ok(())
    }

    #[tracing::instrument(level = "debug", skip(self))]
    async fn delete_private_chat_history(
        &mut self,
        room: SignalingRoomId,
        participant_one: ParticipantId,
        participant_two: ParticipantId,
    ) -> Result<(), SignalingModuleError> {
        state()
            .write()
            .delete_private_chat_history(room, participant_one, participant_two);
        Ok(())
    }

    #[tracing::instrument(level = "debug", skip(self))]
    async fn add_participant_to_group(
        &mut self,
        room: SignalingRoomId,
        group: GroupId,
        participant: ParticipantId,
    ) -> Result<(), SignalingModuleError> {
        state()
            .write()
            .add_participant_to_group(room, group, participant);
        Ok(())
    }

    #[tracing::instrument(level = "debug", skip(self))]
    async fn remove_participant_from_group(
        &mut self,
        room: SignalingRoomId,
        group: GroupId,
        participant: ParticipantId,
    ) {
        state()
            .write()
            .remove_participant_from_group(room, group, participant);
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
    async fn last_seen_global() {
        test_common::last_seen_global(&mut storage().await).await;
    }

    #[tokio::test]
    #[serial]
    async fn last_seen_global_is_personal() {
        test_common::last_seen_global_is_personal(&mut storage().await).await;
    }

    #[tokio::test]
    #[serial]
    async fn last_seen_private() {
        test_common::last_seen_private(&mut storage().await).await;
    }

    #[tokio::test]
    #[serial]
    async fn last_seen_private_is_personal() {
        test_common::last_seen_private_is_personal(&mut storage().await).await;
    }
}
