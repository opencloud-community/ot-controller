// SPDX-FileCopyrightText: OpenTalk GmbH <mail@opentalk.eu>
//
// SPDX-License-Identifier: EUPL-1.2

use std::{
    collections::BTreeSet,
    sync::{Arc, OnceLock},
};

use async_trait::async_trait;
use chrono::{DateTime, Utc};
use either::Either;
use opentalk_signaling_core::{
    LockError, RoomGuard, RoomLocking, SignalingModuleError, SignalingRoomId,
    VolatileStaticMemoryStorage,
};
use opentalk_types_signaling::ParticipantId;
use parking_lot::RwLock;
use snafu::whatever;

use super::memory::MemoryAutomodState;
use crate::storage::{
    automod_storage::{
        AutomodAllowListStorage, AutomodConfigStorage, AutomodHistoryStorage,
        AutomodPlaylistStorage, AutomodSpeakerStorage, Entry, RoomAutomodLock,
    },
    AutomodStorage, StorageConfig,
};

static STATE: OnceLock<Arc<RwLock<MemoryAutomodState>>> = OnceLock::new();

fn state() -> &'static Arc<RwLock<MemoryAutomodState>> {
    STATE.get_or_init(Default::default)
}

#[async_trait(?Send)]
impl AutomodStorage for VolatileStaticMemoryStorage {}

#[async_trait(?Send)]
impl RoomLocking<RoomAutomodLock> for VolatileStaticMemoryStorage {
    async fn lock_room(
        &mut self,
        room: SignalingRoomId,
    ) -> Result<RoomGuard<RoomAutomodLock>, LockError> {
        let lock = state().write().get_room_lock(room);
        let guard = lock.lock_owned().await;

        Ok(RoomGuard {
            room,
            guard: Either::Left(guard),
        })
    }

    async fn unlock_room(
        &mut self,
        RoomGuard { room, guard }: RoomGuard<RoomAutomodLock>,
    ) -> Result<(), LockError> {
        match guard {
            Either::Right(_) => {
                whatever!("Attempted to unlock a redis storage room guard in an in-memory backend")
            }
            Either::Left(guard) => {
                drop(guard);
                state().write().remove_if_unused(room);
            }
        }
        Ok(())
    }
}

#[async_trait(?Send)]
impl AutomodPlaylistStorage for VolatileStaticMemoryStorage {
    #[tracing::instrument(name = "set_playlist", level = "debug", skip(self, playlist))]
    async fn playlist_set(
        &mut self,
        room: SignalingRoomId,
        playlist: &[ParticipantId],
    ) -> Result<(), SignalingModuleError> {
        state().write().playlist_set(room, playlist);

        Ok(())
    }

    #[tracing::instrument(name = "push_playlist", skip(self, participant))]
    async fn playlist_push(
        &mut self,
        room: SignalingRoomId,
        participant: ParticipantId,
    ) -> Result<(), SignalingModuleError> {
        state().write().playlist_push(room, participant);

        Ok(())
    }

    #[tracing::instrument(name = "pop_playlist", level = "debug", skip(self))]
    async fn playlist_pop(
        &mut self,
        room: SignalingRoomId,
    ) -> Result<Option<ParticipantId>, SignalingModuleError> {
        Ok(state().write().playlist_pop(room))
    }

    #[tracing::instrument(name = "get_playlist", level = "debug", skip(self))]
    async fn playlist_get_all(
        &mut self,
        room: SignalingRoomId,
    ) -> Result<Vec<ParticipantId>, SignalingModuleError> {
        Ok(state().read().playlist_get_all(room))
    }

    #[tracing::instrument(name = "remove_from_playlist", level = "debug", skip(self))]
    async fn playlist_remove_first(
        &mut self,
        room: SignalingRoomId,
        participant: ParticipantId,
    ) -> Result<(), SignalingModuleError> {
        state().write().playlist_remove_first(room, participant);
        Ok(())
    }

    #[tracing::instrument(
        name = "remove_all_occurences_from_playlist",
        level = "debug",
        skip(self)
    )]
    async fn playlist_remove_all_occurrences(
        &mut self,
        room: SignalingRoomId,
        participant: ParticipantId,
    ) -> Result<usize, SignalingModuleError> {
        Ok(state()
            .write()
            .playlist_remove_all_occurrences(room, participant))
    }

    #[tracing::instrument(name = "del_playlist", level = "debug", skip(self))]
    async fn playlist_delete(&mut self, room: SignalingRoomId) -> Result<(), SignalingModuleError> {
        state().write().playlist_delete(room);
        Ok(())
    }
}

#[async_trait(?Send)]
impl AutomodAllowListStorage for VolatileStaticMemoryStorage {
    #[tracing::instrument(name = "set_allow_list", skip(self, allow_list))]
    async fn allow_list_set(
        &mut self,
        room: SignalingRoomId,
        allow_list: &[ParticipantId],
    ) -> Result<(), SignalingModuleError> {
        state().write().allow_list_set(room, allow_list);
        Ok(())
    }

    #[tracing::instrument(name = "add_to_allow_list", skip(self, participant))]
    async fn allow_list_add(
        &mut self,
        room: SignalingRoomId,
        participant: ParticipantId,
    ) -> Result<(), SignalingModuleError> {
        state().write().allow_list_add(room, participant);
        Ok(())
    }

    #[tracing::instrument(name = "remove_from_allow_list", skip(self))]
    async fn allow_list_remove(
        &mut self,
        room: SignalingRoomId,
        participant: ParticipantId,
    ) -> Result<usize, SignalingModuleError> {
        Ok(state().write().allow_list_remove(room, participant))
    }

    #[tracing::instrument(name = "random_member_allow_list", skip(self))]
    async fn allow_list_random(
        &mut self,
        room: SignalingRoomId,
    ) -> Result<Option<ParticipantId>, SignalingModuleError> {
        Ok(state().read().allow_list_random(room))
    }

    #[tracing::instrument(skip(self))]
    async fn allow_list_pop_random(
        &mut self,
        room: SignalingRoomId,
    ) -> Result<Option<ParticipantId>, SignalingModuleError> {
        Ok(state().write().allow_list_pop_random(room))
    }

    #[tracing::instrument(skip(self))]
    async fn allow_list_contains(
        &mut self,
        room: SignalingRoomId,
        participant: ParticipantId,
    ) -> Result<bool, SignalingModuleError> {
        Ok(state().read().allow_list_contains(room, participant))
    }

    #[tracing::instrument(name = "get_all_allow_list", skip(self))]
    async fn allow_list_get_all(
        &mut self,
        room: SignalingRoomId,
    ) -> Result<BTreeSet<ParticipantId>, SignalingModuleError> {
        Ok(state().read().allow_list_get_all(room))
    }

    #[tracing::instrument(name = "del_allow_list", skip(self))]
    async fn allow_list_delete(
        &mut self,
        room: SignalingRoomId,
    ) -> Result<(), SignalingModuleError> {
        state().write().allow_list_delete(room);
        Ok(())
    }
}

#[async_trait(?Send)]
impl AutomodConfigStorage for VolatileStaticMemoryStorage {
    #[tracing::instrument(name = "set_config", level = "debug", skip(self, config))]
    async fn config_set(
        &mut self,
        room: SignalingRoomId,
        config: StorageConfig,
    ) -> Result<(), SignalingModuleError> {
        state().write().config_set(room, config);
        Ok(())
    }

    #[tracing::instrument(name = "get_config", level = "debug", skip(self))]
    async fn config_get(
        &mut self,
        room: SignalingRoomId,
    ) -> Result<Option<StorageConfig>, SignalingModuleError> {
        Ok(state().read().config_get(room))
    }

    #[tracing::instrument(name = "del_config", level = "debug", skip(self))]
    async fn config_delete(&mut self, room: SignalingRoomId) -> Result<(), SignalingModuleError> {
        state().write().config_delete(room);
        Ok(())
    }

    #[tracing::instrument(name = "exists_config", level = "debug", skip(self))]
    async fn config_exists(&mut self, room: SignalingRoomId) -> Result<bool, SignalingModuleError> {
        Ok(state().read().config_exists(room))
    }
}

#[async_trait(?Send)]
impl AutomodSpeakerStorage for VolatileStaticMemoryStorage {
    #[tracing::instrument(name = "get_speaker", level = "debug", skip(self))]
    async fn speaker_get(
        &mut self,
        room: SignalingRoomId,
    ) -> Result<Option<ParticipantId>, SignalingModuleError> {
        Ok(state().read().speaker_get(room))
    }

    #[tracing::instrument(name = "set_speaker", level = "debug", skip(self))]
    async fn speaker_set(
        &mut self,
        room: SignalingRoomId,
        participant: ParticipantId,
    ) -> Result<Option<ParticipantId>, SignalingModuleError> {
        Ok(state().write().speaker_set(room, participant))
    }

    #[tracing::instrument(name = "del_speaker", level = "debug", skip(self))]
    async fn speaker_delete(
        &mut self,
        room: SignalingRoomId,
    ) -> Result<Option<ParticipantId>, SignalingModuleError> {
        Ok(state().write().speaker_delete(room))
    }
}

#[async_trait(?Send)]
impl AutomodHistoryStorage for VolatileStaticMemoryStorage {
    #[tracing::instrument(name = "add_history", level = "debug", skip(self, entry))]
    async fn history_add(
        &mut self,
        room: SignalingRoomId,
        entry: Entry,
    ) -> Result<(), SignalingModuleError> {
        state().write().history_add(room, entry);
        Ok(())
    }

    #[tracing::instrument(name = "get_history", level = "debug", skip(self))]
    async fn history_get(
        &mut self,
        room: SignalingRoomId,
        since: DateTime<Utc>,
    ) -> Result<Vec<ParticipantId>, SignalingModuleError> {
        Ok(state().read().history_get(room, since))
    }

    #[tracing::instrument(name = "del_history", level = "debug", skip(self))]
    async fn history_delete(&mut self, room: SignalingRoomId) -> Result<(), SignalingModuleError> {
        state().write().history_delete(room);
        Ok(())
    }

    #[cfg(test)]
    async fn history_get_entries(
        &mut self,
        room: SignalingRoomId,
        since: DateTime<Utc>,
    ) -> Result<Vec<Entry>, SignalingModuleError> {
        Ok(state().read().history_get_entries(room, since))
    }
}

#[cfg(test)]
pub(crate) mod tests {
    use opentalk_signaling_core::VolatileStaticMemoryStorage;
    use serial_test::serial;

    use super::state;
    use crate::storage::test_common;

    pub(crate) fn reset_state() {
        state().write().reset();
    }

    fn storage() -> VolatileStaticMemoryStorage {
        state().write().reset();
        VolatileStaticMemoryStorage
    }

    #[tokio::test]
    #[serial]
    async fn playlist() {
        test_common::playlist(&mut storage()).await
    }

    #[tokio::test]
    #[serial]
    async fn playlist_remove_first() {
        test_common::playlist_remove_first(&mut storage()).await
    }

    #[tokio::test]
    #[serial]
    async fn allow_list() {
        test_common::allow_list(&mut storage()).await
    }

    #[tokio::test]
    #[serial]
    async fn storage_config() {
        test_common::storage_config(&mut storage()).await
    }

    #[tokio::test]
    #[serial]
    async fn speaker() {
        test_common::speaker(&mut storage()).await
    }

    #[tokio::test]
    #[serial]
    async fn history() {
        test_common::history(&mut storage()).await
    }

    #[tokio::test]
    #[serial]
    async fn history_repeated_speaker() {
        test_common::history_repeated_speaker(&mut storage()).await
    }
}
