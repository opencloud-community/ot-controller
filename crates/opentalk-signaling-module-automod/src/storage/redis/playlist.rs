// SPDX-FileCopyrightText: OpenTalk GmbH <mail@opentalk.eu>
//
// SPDX-License-Identifier: EUPL-1.2

//! Ordered list of participants used by the `Playlist` selection strategy.
//!
//! Depending on the selection strategy:
//!
//! - `none`, `random` or `nomination`: The playlist does not get used by these strategies.
//!
//! - `playlist` The playlist is a ordered list of participants which will get used to select
//!   the next participant when yielding. It is also used as a pool to select participants
//!   randomly from (moderator command `Select`).

use async_trait::async_trait;
use opentalk_signaling_core::{RedisConnection, RedisSnafu, SignalingModuleError, SignalingRoomId};
use opentalk_types_signaling::ParticipantId;
use redis::AsyncCommands;
use redis_args::ToRedisArgs;
use snafu::ResultExt;

use crate::storage::automod_storage::AutomodPlaylistStorage;

#[async_trait(?Send)]
impl AutomodPlaylistStorage for RedisConnection {
    #[tracing::instrument(name = "set_playlist", level = "debug", skip(self, playlist))]
    async fn playlist_set(
        &mut self,
        room: SignalingRoomId,
        playlist: &[ParticipantId],
    ) -> Result<(), SignalingModuleError> {
        self.del::<_, ()>(RoomAutomodPlaylist { room })
            .await
            .context(RedisSnafu {
                message: "Failed to delete playlist to later reinsert it",
            })?;

        if !playlist.is_empty() {
            self.rpush(RoomAutomodPlaylist { room }, playlist)
                .await
                .context(RedisSnafu {
                    message: "Failed to insert new list",
                })
        } else {
            Ok(())
        }
    }

    #[tracing::instrument(name = "push_playlist", skip(self, participant_id))]
    async fn playlist_push(
        &mut self,
        room: SignalingRoomId,
        participant_id: ParticipantId,
    ) -> Result<(), SignalingModuleError> {
        self.rpush(RoomAutomodPlaylist { room }, participant_id)
            .await
            .context(RedisSnafu {
                message: "Failed to push participant_id to playlist",
            })
    }

    #[tracing::instrument(name = "pop_playlist", level = "debug", skip(self))]
    async fn playlist_pop(
        &mut self,
        room: SignalingRoomId,
    ) -> Result<Option<ParticipantId>, SignalingModuleError> {
        self.lpop(RoomAutomodPlaylist { room }, None)
            .await
            .context(RedisSnafu {
                message: "Failed to pop playlist",
            })
    }

    #[tracing::instrument(name = "get_playlist", level = "debug", skip(self))]
    async fn playlist_get_all(
        &mut self,
        room: SignalingRoomId,
    ) -> Result<Vec<ParticipantId>, SignalingModuleError> {
        self.lrange(RoomAutomodPlaylist { room }, 0, -1)
            .await
            .context(RedisSnafu {
                message: "Failed to get_all playlist",
            })
    }

    #[tracing::instrument(name = "remove_from_playlist", level = "debug", skip(self))]
    async fn playlist_remove_first(
        &mut self,
        room: SignalingRoomId,
        participant: ParticipantId,
    ) -> Result<(), SignalingModuleError> {
        self.lrem(RoomAutomodPlaylist { room }, 1, participant)
            .await
            .context(RedisSnafu {
                message: "Failed to remove participant from playlist",
            })
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
        self.lrem(RoomAutomodPlaylist { room }, 0, participant)
            .await
            .context(RedisSnafu {
                message: "Failed to remove all occurrences of participant from playlist",
            })
    }

    #[tracing::instrument(name = "del_playlist", level = "debug", skip(self))]
    async fn playlist_delete(&mut self, room: SignalingRoomId) -> Result<(), SignalingModuleError> {
        self.del(RoomAutomodPlaylist { room })
            .await
            .context(RedisSnafu {
                message: "Failed to del playlist",
            })
    }
}

/// Typed key to the automod playlist
#[derive(ToRedisArgs)]
#[to_redis_args(fmt = "opentalk-signaling:room={room}:automod:playlist")]
struct RoomAutomodPlaylist {
    room: SignalingRoomId,
}
