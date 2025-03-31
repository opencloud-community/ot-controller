// SPDX-FileCopyrightText: OpenTalk GmbH <mail@opentalk.eu>
//
// SPDX-License-Identifier: EUPL-1.2

//! History of all speakers inside a room session. The history lives over multiple automod sessions
//! until the room is being torn down.
//!
//! Events are stored inside a redis sorted set.
//! The score of the events is milliseconds of the timestamp.
//!
//! An event is recorded everytime a participant gains and loses its speaker status.

use async_trait::async_trait;
use chrono::{DateTime, Utc};
use opentalk_signaling_core::{RedisConnection, RedisSnafu, SignalingModuleError, SignalingRoomId};
use opentalk_types_signaling::ParticipantId;
use redis::AsyncCommands;
use redis_args::ToRedisArgs;
use snafu::ResultExt;

use crate::storage::{
    automod_storage::{AutomodHistoryStorage, Entry},
    EntryKind,
};

#[async_trait(?Send)]
impl AutomodHistoryStorage for RedisConnection {
    #[tracing::instrument(name = "add_history", level = "debug", skip(self, entry))]
    async fn history_add(
        &mut self,
        room: SignalingRoomId,
        entry: Entry,
    ) -> Result<(), SignalingModuleError> {
        self.zadd(
            RoomAutomodHistory { room },
            entry,
            entry.timestamp.timestamp_millis(),
        )
        .await
        .context(RedisSnafu {
            message: "Failed to add history entry",
        })
    }

    #[tracing::instrument(name = "get_history", level = "debug", skip(self))]
    async fn history_get(
        &mut self,
        room: SignalingRoomId,
        since: DateTime<Utc>,
    ) -> Result<Vec<ParticipantId>, SignalingModuleError> {
        let entries: Vec<Entry> = self
            .zrangebyscore(
                RoomAutomodHistory { room },
                since.timestamp_millis(),
                "+inf",
            )
            .await
            .context(RedisSnafu {
                message: "Failed to get history entries",
            })?;

        let participants = entries
            .into_iter()
            .filter(|entry| matches!(entry.kind, EntryKind::Start))
            .map(|entry| entry.participant)
            .collect();

        Ok(participants)
    }

    #[tracing::instrument(name = "del_history", level = "debug", skip(self))]
    async fn history_delete(&mut self, room: SignalingRoomId) -> Result<(), SignalingModuleError> {
        self.del(RoomAutomodHistory { room })
            .await
            .context(RedisSnafu {
                message: "Failed to del history",
            })
    }

    #[cfg(test)]
    async fn history_get_entries(
        &mut self,
        room: SignalingRoomId,
        since: DateTime<Utc>,
    ) -> Result<Vec<Entry>, SignalingModuleError> {
        self.zrangebyscore(
            RoomAutomodHistory { room },
            since.timestamp_millis(),
            "+inf",
        )
        .await
        .context(RedisSnafu {
            message: "Failed to get history entries",
        })
    }
}

/// Typed key to the automod history
#[derive(ToRedisArgs)]
#[to_redis_args(fmt = "opentalk-signaling:room={room}:automod:history")]
struct RoomAutomodHistory {
    room: SignalingRoomId,
}
