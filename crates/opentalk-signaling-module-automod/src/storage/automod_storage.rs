// SPDX-FileCopyrightText: OpenTalk GmbH <mail@opentalk.eu>
//
// SPDX-License-Identifier: EUPL-1.2

use std::collections::BTreeSet;

use async_trait::async_trait;
use chrono::{DateTime, Utc};
use opentalk_signaling_core::{
    RoomLocking, SignalingModuleError, SignalingRoomId,
    control::storage::ControlStorageParticipantSet,
};
use opentalk_types_signaling::ParticipantId;
use redis_args::{FromRedisValue, ToRedisArgs};
use serde::{Deserialize, Serialize};

use crate::storage::StorageConfig;

#[async_trait(?Send)]
pub(crate) trait AutomodStorage:
    RoomLocking<RoomAutomodLock>
    + AutomodPlaylistStorage
    + AutomodAllowListStorage
    + AutomodConfigStorage
    + AutomodSpeakerStorage
    + AutomodHistoryStorage
    + ControlStorageParticipantSet
{
}

#[async_trait(?Send)]
pub(crate) trait AutomodPlaylistStorage {
    /// Set the playlist. If the `playlist` parameter is empty the old one will still be removed.
    async fn playlist_set(
        &mut self,
        room: SignalingRoomId,
        playlist: &[ParticipantId],
    ) -> Result<(), SignalingModuleError>;

    /// Insert the given participant at the end of the playlist
    async fn playlist_push(
        &mut self,
        room: SignalingRoomId,
        participant_id: ParticipantId,
    ) -> Result<(), SignalingModuleError>;

    /// Get and remove the next participant from the playlist
    async fn playlist_pop(
        &mut self,
        room: SignalingRoomId,
    ) -> Result<Option<ParticipantId>, SignalingModuleError>;

    /// Returns the playlist in a Vec.
    async fn playlist_get_all(
        &mut self,
        room: SignalingRoomId,
    ) -> Result<Vec<ParticipantId>, SignalingModuleError>;

    /// Remove first occurrence of `participant` from the playlist.
    async fn playlist_remove_first(
        &mut self,
        room: SignalingRoomId,
        participant: ParticipantId,
    ) -> Result<(), SignalingModuleError>;

    /// Remove all occurrences of `participant` from the playlist.
    async fn playlist_remove_all_occurrences(
        &mut self,
        room: SignalingRoomId,
        participant: ParticipantId,
    ) -> Result<usize, SignalingModuleError>;

    /// Delete the complete playlist.
    async fn playlist_delete(&mut self, room: SignalingRoomId) -> Result<(), SignalingModuleError>;
}

#[async_trait(?Send)]
pub(crate) trait AutomodAllowListStorage {
    /// Override the current allow_list with the given one. If the `allow_list` parameter is empty,
    /// the entry will just be deleted.
    async fn allow_list_set(
        &mut self,
        room: SignalingRoomId,
        allow_list: &[ParticipantId],
    ) -> Result<(), SignalingModuleError>;

    /// Add the given participant to the allow list.
    async fn allow_list_add(
        &mut self,
        room: SignalingRoomId,
        participant: ParticipantId,
    ) -> Result<(), SignalingModuleError>;

    /// Remove the given participant from the allow_list
    async fn allow_list_remove(
        &mut self,
        room: SignalingRoomId,
        participant: ParticipantId,
    ) -> Result<usize, SignalingModuleError>;

    /// Get a random `participant` from the allow_list. Will return `None` if the allow_list if empty.
    async fn allow_list_random(
        &mut self,
        room: SignalingRoomId,
    ) -> Result<Option<ParticipantId>, SignalingModuleError>;

    /// Pop a random `participant` from the allow_list. Will return `None` if the allow_list if empty.
    async fn allow_list_pop_random(
        &mut self,
        room: SignalingRoomId,
    ) -> Result<Option<ParticipantId>, SignalingModuleError>;

    /// Check if the given `participant` is allowed by the `allow_list`.
    async fn allow_list_contains(
        &mut self,
        room: SignalingRoomId,
        participant: ParticipantId,
    ) -> Result<bool, SignalingModuleError>;

    /// Return all members of the `allow_list`.
    async fn allow_list_get_all(
        &mut self,
        room: SignalingRoomId,
    ) -> Result<BTreeSet<ParticipantId>, SignalingModuleError>;

    /// Delete the `allow_list`.
    async fn allow_list_delete(
        &mut self,
        room: SignalingRoomId,
    ) -> Result<(), SignalingModuleError>;
}

#[async_trait(?Send)]
pub(crate) trait AutomodConfigStorage {
    /// Set the current config.
    async fn config_set(
        &mut self,
        room: SignalingRoomId,
        config: StorageConfig,
    ) -> Result<(), SignalingModuleError>;

    /// Get the current config, if any is set.
    ///
    /// If it returns `Some`, one must assume the automod is active.
    async fn config_get(
        &mut self,
        room: SignalingRoomId,
    ) -> Result<Option<StorageConfig>, SignalingModuleError>;

    /// Delete the config.
    async fn config_delete(&mut self, room: SignalingRoomId) -> Result<(), SignalingModuleError>;

    /// Query for the current config, if any is set.
    ///
    /// If it returns `true`, one can assume the automod is active.
    async fn config_exists(&mut self, room: SignalingRoomId) -> Result<bool, SignalingModuleError>;
}

#[async_trait(?Send)]
pub(crate) trait AutomodSpeakerStorage {
    /// Get the current speaker. Returns [`None`] if there is no active speaker.
    async fn speaker_get(
        &mut self,
        room: SignalingRoomId,
    ) -> Result<Option<ParticipantId>, SignalingModuleError>;

    /// Sets the new current speaker and returns the old one if it was set
    async fn speaker_set(
        &mut self,
        room: SignalingRoomId,
        participant: ParticipantId,
    ) -> Result<Option<ParticipantId>, SignalingModuleError>;

    /// Delete the current speaker and return if there was any speaker.
    async fn speaker_delete(
        &mut self,
        room: SignalingRoomId,
    ) -> Result<Option<ParticipantId>, SignalingModuleError>;
}

#[async_trait(?Send)]
pub(crate) trait AutomodHistoryStorage {
    /// Adds the given entry to the history
    async fn history_add(
        &mut self,
        room: SignalingRoomId,
        entry: Entry,
    ) -> Result<(), SignalingModuleError>;

    /// Get a ordered list of participants which appear in the history after the given `since` parameter
    /// timestamp.
    async fn history_get(
        &mut self,
        room: SignalingRoomId,
        since: DateTime<Utc>,
    ) -> Result<Vec<ParticipantId>, SignalingModuleError>;

    /// Delete the history.
    async fn history_delete(&mut self, room: SignalingRoomId) -> Result<(), SignalingModuleError>;

    #[cfg(test)]
    async fn history_get_entries(
        &mut self,
        room: SignalingRoomId,
        since: DateTime<Utc>,
    ) -> Result<Vec<Entry>, SignalingModuleError>;
}

/// Entry inside the automod history
#[derive(
    Debug,
    Clone,
    PartialEq,
    Eq,
    PartialOrd,
    Ord,
    Copy,
    Serialize,
    Deserialize,
    ToRedisArgs,
    FromRedisValue,
)]
#[to_redis_args(serde)]
#[from_redis_value(serde)]
pub struct Entry {
    pub timestamp: DateTime<Utc>,
    pub participant: ParticipantId,
    pub kind: EntryKind,
}

/// The kind of history entry.
#[derive(Debug, Clone, Copy, Serialize, Deserialize, PartialEq, Eq, PartialOrd, Ord)]
pub enum EntryKind {
    /// Participant gained its speaker status.
    Start,

    /// Participant lost its speaker status.
    Stop,
}

impl Entry {
    /// Creates a new Start-Entry with the current timestamp and the given participant.
    pub fn start(participant: ParticipantId) -> Self {
        Self {
            timestamp: Utc::now(),
            participant,
            kind: EntryKind::Start,
        }
    }

    /// Creates a new Stop-Entry with the current timestamp and the given participant.
    pub fn stop(participant: ParticipantId) -> Self {
        Self {
            timestamp: Utc::now(),
            participant,
            kind: EntryKind::Stop,
        }
    }
}

/// Typed key to the automod lock
#[derive(ToRedisArgs)]
#[to_redis_args(fmt = "opentalk-signaling:room={room}:automod:lock")]
pub struct RoomAutomodLock {
    room: SignalingRoomId,
}

impl From<SignalingRoomId> for RoomAutomodLock {
    fn from(room: SignalingRoomId) -> Self {
        Self { room }
    }
}
