// SPDX-FileCopyrightText: OpenTalk GmbH <mail@opentalk.eu>
//
// SPDX-License-Identifier: EUPL-1.2

use std::{
    collections::{BTreeMap, BTreeSet, btree_map},
    sync::{Arc, Weak},
};

use chrono::{DateTime, Utc};
use futures::lock::Mutex;
use opentalk_signaling_core::SignalingRoomId;
use opentalk_types_signaling::ParticipantId;
use rand::seq::IteratorRandom;

use crate::storage::{Entry, EntryKind, StorageConfig, automod_storage::RoomAutomodLock};

#[derive(Debug, Clone, Default)]
pub(crate) struct MemoryAutomodState {
    playlists: BTreeMap<SignalingRoomId, Vec<ParticipantId>>,
    allow_lists: BTreeMap<SignalingRoomId, BTreeSet<ParticipantId>>,
    configs: BTreeMap<SignalingRoomId, StorageConfig>,
    speakers: BTreeMap<SignalingRoomId, ParticipantId>,
    histories: BTreeMap<SignalingRoomId, BTreeSet<Entry>>,
    locks: BTreeMap<SignalingRoomId, Weak<Mutex<RoomAutomodLock>>>,
}

impl MemoryAutomodState {
    #[cfg(test)]
    pub(super) fn reset(&mut self) {
        *self = Self::default();
    }

    pub(crate) fn get_room_lock(&mut self, room: SignalingRoomId) -> Arc<Mutex<RoomAutomodLock>> {
        match self.locks.entry(room) {
            btree_map::Entry::Vacant(entry) => {
                let mutex = Arc::new(Mutex::new(RoomAutomodLock::from(room)));
                entry.insert(Arc::downgrade(&mutex));
                mutex
            }
            btree_map::Entry::Occupied(mut entry) => {
                if let Some(mutex) = entry.get().upgrade() {
                    mutex
                } else {
                    let mutex = Arc::new(Mutex::new(RoomAutomodLock::from(room)));
                    entry.insert(Arc::downgrade(&mutex));
                    mutex
                }
            }
        }
    }

    pub(crate) fn remove_if_unused(&mut self, room: SignalingRoomId) {
        if let Some(lock) = self.locks.get(&room) {
            if lock.strong_count() == 0 {
                self.locks.remove(&room);
            }
        }
    }

    pub(crate) fn playlist_set(&mut self, room: SignalingRoomId, playlist: &[ParticipantId]) {
        self.playlists.insert(room, playlist.to_vec());
    }

    pub(crate) fn playlist_push(&mut self, room: SignalingRoomId, participant: ParticipantId) {
        let playlist = self.playlists.entry(room).or_default();
        playlist.push(participant);
    }

    pub(crate) fn playlist_pop(&mut self, room: SignalingRoomId) -> Option<ParticipantId> {
        self.playlists
            .get_mut(&room)
            .filter(|playlist| !playlist.is_empty())
            .map(|playlist| playlist.remove(0))
    }

    pub(crate) fn playlist_get_all(&self, room: SignalingRoomId) -> Vec<ParticipantId> {
        self.playlists.get(&room).cloned().unwrap_or_default()
    }

    pub(crate) fn playlist_remove_first(
        &mut self,
        room: SignalingRoomId,
        to_be_removed: ParticipantId,
    ) {
        let Some(playlist) = self.playlists.get_mut(&room) else {
            return;
        };
        let Some(index) = playlist
            .iter()
            .position(|&participant| participant == to_be_removed)
        else {
            return;
        };
        playlist.remove(index);
    }

    pub(crate) fn playlist_remove_all_occurrences(
        &mut self,
        room: SignalingRoomId,
        to_be_removed: ParticipantId,
    ) -> usize {
        let Some(playlist) = self.playlists.get_mut(&room) else {
            return 0;
        };
        let old_len = playlist.len();
        playlist.retain(|participant| participant != &to_be_removed);
        old_len - playlist.len()
    }

    pub(crate) fn playlist_delete(&mut self, room: SignalingRoomId) {
        self.playlists.remove(&room);
    }

    pub(crate) fn allow_list_set(&mut self, room: SignalingRoomId, new: &[ParticipantId]) {
        let allow_list = self.allow_lists.entry(room).or_default();
        allow_list.clear();
        allow_list.extend(new)
    }

    pub(crate) fn allow_list_add(&mut self, room: SignalingRoomId, participant: ParticipantId) {
        self.allow_lists
            .entry(room)
            .or_default()
            .insert(participant);
    }

    pub(crate) fn allow_list_remove(
        &mut self,
        room: SignalingRoomId,
        participant: ParticipantId,
    ) -> usize {
        if self
            .allow_lists
            .entry(room)
            .or_default()
            .remove(&participant)
        {
            1
        } else {
            0
        }
    }

    pub(crate) fn allow_list_random(&self, room: SignalingRoomId) -> Option<ParticipantId> {
        let mut rng = rand::rng();
        self.allow_lists
            .get(&room)
            .and_then(|allow_list| allow_list.iter().choose(&mut rng))
            .copied()
    }

    pub(crate) fn allow_list_pop_random(&mut self, room: SignalingRoomId) -> Option<ParticipantId> {
        let mut rng = rand::rng();
        let allow_list = self.allow_lists.get_mut(&room)?;
        let participant = *allow_list.iter().choose(&mut rng)?;
        if allow_list.remove(&participant) {
            Some(participant)
        } else {
            None
        }
    }

    pub(crate) fn allow_list_contains(
        &self,
        room: SignalingRoomId,
        participant: ParticipantId,
    ) -> bool {
        self.allow_lists
            .get(&room)
            .map(|allow_list| allow_list.contains(&participant))
            .unwrap_or(false)
    }

    pub(crate) fn allow_list_get_all(&self, room: SignalingRoomId) -> BTreeSet<ParticipantId> {
        self.allow_lists.get(&room).cloned().unwrap_or_default()
    }

    pub(crate) fn allow_list_delete(&mut self, room: SignalingRoomId) {
        self.allow_lists.remove(&room);
    }

    pub(crate) fn config_set(&mut self, room: SignalingRoomId, config: StorageConfig) {
        self.configs.insert(room, config);
    }

    pub(crate) fn config_get(&self, room: SignalingRoomId) -> Option<StorageConfig> {
        self.configs.get(&room).cloned()
    }

    pub(crate) fn config_delete(&mut self, room: SignalingRoomId) {
        self.configs.remove(&room);
    }

    pub(crate) fn config_exists(&self, room: SignalingRoomId) -> bool {
        self.configs.contains_key(&room)
    }

    pub(crate) fn speaker_get(&self, room: SignalingRoomId) -> Option<ParticipantId> {
        self.speakers.get(&room).copied()
    }

    pub(crate) fn speaker_set(
        &mut self,
        room: SignalingRoomId,
        participant: ParticipantId,
    ) -> Option<ParticipantId> {
        self.speakers.insert(room, participant)
    }

    pub(crate) fn speaker_delete(&mut self, room: SignalingRoomId) -> Option<ParticipantId> {
        self.speakers.remove(&room)
    }

    pub(crate) fn history_add(&mut self, room: SignalingRoomId, entry: Entry) {
        let history = self.histories.entry(room).or_default();
        history.insert(entry);
    }

    pub(crate) fn history_get(
        &self,
        room: SignalingRoomId,
        since: DateTime<Utc>,
    ) -> Vec<ParticipantId> {
        self.history_iter_entries(room, since)
            //FIXME: recording start and stop seems redundant, we never expose the duration.
            .filter(|entry| entry.kind == EntryKind::Start)
            .map(|entry| entry.participant)
            .collect()
    }

    pub(crate) fn history_delete(&mut self, room: SignalingRoomId) {
        self.histories.remove(&room);
    }

    #[cfg(test)]
    pub(crate) fn history_get_entries(
        &self,
        room: SignalingRoomId,
        since: DateTime<Utc>,
    ) -> Vec<Entry> {
        self.history_iter_entries(room, since).collect()
    }

    fn history_iter_entries(
        &self,
        room: SignalingRoomId,
        since: DateTime<Utc>,
    ) -> impl Iterator<Item = Entry> + '_ {
        self.histories
            .get(&room)
            .map(BTreeSet::iter)
            .unwrap_or_default()
            .filter(move |entry| entry.timestamp >= since)
            .copied()
    }
}
