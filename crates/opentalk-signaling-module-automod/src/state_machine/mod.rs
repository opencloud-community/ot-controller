// SPDX-FileCopyrightText: OpenTalk GmbH <mail@opentalk.eu>
//
// SPDX-License-Identifier: EUPL-1.2

//! Contains and reexports all functions required to select a speaker from the state machine.
//!
//! The state machine stores its state complete exclusively inside Redis. See the `storage` module
//! for more information.

use opentalk_signaling_core::{SignalingModuleError, SignalingRoomId};
use opentalk_types_signaling::ParticipantId;
use opentalk_types_signaling_automod::config::SelectionStrategy;
use snafu::Snafu;

use crate::{
    AutomodStorage, exchange,
    storage::{Entry, StorageConfig},
};

mod next;
mod random;

pub use next::select_next;
pub use random::select_random;

/// Error returned by the state machine
#[derive(Debug, Snafu)]
pub enum Error {
    /// The user made an invalid selection, either the participant does not exist or isn't eligible
    /// for selection.
    #[snafu(display("invalid selection"))]
    InvalidSelection,

    /// A general fatal error has occurred (bug or IO)
    #[snafu(whatever)]
    Fatal {
        message: String,

        #[snafu(source(from(Box<dyn std::error::Error + Send + Sync>, Some)))]
        source: Option<Box<dyn std::error::Error + Send + Sync>>,
    },
}

impl From<SignalingModuleError> for Error {
    fn from(e: SignalingModuleError) -> Self {
        Self::Fatal {
            message: "Signaling module error".to_owned(),
            source: Some(e.into()),
        }
    }
}

#[derive(Debug, PartialEq, Eq)]
pub enum StateMachineOutput {
    SpeakerUpdate(exchange::SpeakerUpdate),
    StartAnimation(exchange::StartAnimation),
}

pub fn map_select_unchecked(
    output: Result<Option<exchange::SpeakerUpdate>, Error>,
) -> Result<Option<StateMachineOutput>, Error> {
    output.map(|opt| opt.map(StateMachineOutput::SpeakerUpdate))
}

/// Selects the given participant (or None) as the current speaker and generates the appropriate
/// [`SpeakerUpdate`] if necessary.
/// Does not check if the participant exists or is even eligible to be speaker.
pub async fn select_unchecked(
    storage: &mut dyn AutomodStorage,
    room: SignalingRoomId,
    config: &StorageConfig,
    participant: Option<ParticipantId>,
) -> Result<Option<exchange::SpeakerUpdate>, Error> {
    let previous = if let Some(participant) = participant {
        storage.speaker_set(room, participant).await?
    } else {
        storage.speaker_delete(room).await?
    };

    if previous.is_none() && participant.is_none() {
        // nothing changed, return early
        return Ok(None);
    }

    // If there was a previous speaker add stop event to history
    if let Some(previous) = previous {
        storage.history_add(room, Entry::stop(previous)).await?;
    }

    // If there is a new speaker add start event to history
    if let Some(participant) = participant {
        storage.history_add(room, Entry::start(participant)).await?;
    }

    let history = storage.history_get(room, config.started).await?;

    let remaining = match config.parameter.selection_strategy {
        SelectionStrategy::None | SelectionStrategy::Random | SelectionStrategy::Nomination => {
            Some(
                storage
                    .allow_list_get_all(room)
                    .await?
                    .into_iter()
                    .collect(),
            )
        }
        SelectionStrategy::Playlist => Some(storage.playlist_get_all(room).await?),
    };

    Ok(Some(exchange::SpeakerUpdate {
        speaker: participant,
        history: Some(history).filter(|history| !history.is_empty()),
        remaining,
    }))
}

#[cfg(test)]
mod test {
    use std::time::{Duration, SystemTime};

    use chrono::{DateTime, Utc};
    use opentalk_signaling_core::{RedisConnection, SignalingRoomId, VolatileStaticMemoryStorage};
    use rand::{SeedableRng, rngs::StdRng};
    use redis::aio::ConnectionManager;

    use crate::storage::reset_memory_state;

    pub const ROOM: SignalingRoomId = SignalingRoomId::nil();

    pub async fn setup_redis() -> RedisConnection {
        let redis_url =
            std::env::var("REDIS_ADDR").unwrap_or_else(|_| "redis://0.0.0.0:6379/".to_owned());
        let redis = redis::Client::open(redis_url).expect("Invalid redis url");

        let mut mgr = ConnectionManager::new(redis).await.unwrap();

        redis::cmd("FLUSHALL").exec_async(&mut mgr).await.unwrap();

        RedisConnection::new(mgr)
    }

    pub async fn setup_memory() -> VolatileStaticMemoryStorage {
        reset_memory_state();
        VolatileStaticMemoryStorage
    }

    pub fn rng() -> StdRng {
        StdRng::seed_from_u64(0)
    }

    pub fn unix_epoch(secs: u64) -> DateTime<Utc> {
        DateTime::from(SystemTime::UNIX_EPOCH + Duration::from_secs(secs))
    }
}
