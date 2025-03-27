// SPDX-FileCopyrightText: OpenTalk GmbH <mail@opentalk.eu>
//
// SPDX-License-Identifier: EUPL-1.2

//! List of participants used by selection_strategies to help decide who to select.
//!
//! Depending on the selection strategy:
//!
//! - `none`, `random` or `nomination`: The allow_list acts as pool of participants which can
//!   be selected (by nomination or randomly etc).
//!
//! - `playlist` The allow_list does not get used by this strategy.
// TODO: Playlist mode will use this to filter which participants can add themself to the playlist via hand-raise

use std::collections::BTreeSet;

use async_trait::async_trait;
use opentalk_signaling_core::{RedisConnection, RedisSnafu, SignalingModuleError, SignalingRoomId};
use opentalk_types_signaling::ParticipantId;
use redis::AsyncCommands;
use redis_args::ToRedisArgs;
use snafu::ResultExt;

use crate::storage::automod_storage::AutomodAllowListStorage;

#[async_trait(?Send)]
impl AutomodAllowListStorage for RedisConnection {
    #[tracing::instrument(name = "set_allow_list", skip(self, allow_list))]
    async fn allow_list_set(
        &mut self,
        room: SignalingRoomId,
        allow_list: &[ParticipantId],
    ) -> Result<(), SignalingModuleError> {
        self.del::<_, ()>(RoomAutomodAllowList { room })
            .await
            .context(RedisSnafu {
                message: "Failed to delete playlist to later reinsert it",
            })?;

        if !allow_list.is_empty() {
            self.sadd(RoomAutomodAllowList { room }, allow_list)
                .await
                .context(RedisSnafu {
                    message: "Failed to insert new allow_list set",
                })
        } else {
            Ok(())
        }
    }

    #[tracing::instrument(name = "add_to_allow_list", skip(self, participant))]
    async fn allow_list_add(
        &mut self,
        room: SignalingRoomId,
        participant: ParticipantId,
    ) -> Result<(), SignalingModuleError> {
        self.sadd(RoomAutomodAllowList { room }, participant)
            .await
            .context(RedisSnafu {
                message: "Failed to add participant_id to allow_list",
            })
    }

    #[tracing::instrument(name = "remove_from_allow_list", skip(self))]
    async fn allow_list_remove(
        &mut self,
        room: SignalingRoomId,
        participant: ParticipantId,
    ) -> Result<usize, SignalingModuleError> {
        self.srem(RoomAutomodAllowList { room }, participant)
            .await
            .context(RedisSnafu {
                message: "Failed to remove participant from allow_list",
            })
    }

    #[tracing::instrument(name = "random_member_allow_list", skip(self))]
    async fn allow_list_random(
        &mut self,
        room: SignalingRoomId,
    ) -> Result<Option<ParticipantId>, SignalingModuleError> {
        self.srandmember(RoomAutomodAllowList { room })
            .await
            .context(RedisSnafu {
                message: "Failed to get random member from allow list",
            })
    }

    #[tracing::instrument(skip(self))]
    async fn allow_list_pop_random(
        &mut self,
        room: SignalingRoomId,
    ) -> Result<Option<ParticipantId>, SignalingModuleError> {
        self.spop(RoomAutomodAllowList { room })
            .await
            .context(RedisSnafu {
                message: "Failed to pop random member from allow list",
            })
    }

    #[tracing::instrument(skip(self))]
    async fn allow_list_contains(
        &mut self,
        room: SignalingRoomId,
        participant: ParticipantId,
    ) -> Result<bool, SignalingModuleError> {
        self.sismember(RoomAutomodAllowList { room }, participant)
            .await
            .context(RedisSnafu {
                message: "Failed to check if participant is inside allow_list",
            })
    }

    #[tracing::instrument(name = "get_all_allow_list", skip(self))]
    async fn allow_list_get_all(
        &mut self,
        room: SignalingRoomId,
    ) -> Result<BTreeSet<ParticipantId>, SignalingModuleError> {
        self.smembers(RoomAutomodAllowList { room })
            .await
            .context(RedisSnafu {
                message: "Failed to get random member from allow list",
            })
    }

    #[tracing::instrument(name = "del_allow_list", skip(self))]
    async fn allow_list_delete(
        &mut self,
        room: SignalingRoomId,
    ) -> Result<(), SignalingModuleError> {
        self.del(RoomAutomodAllowList { room })
            .await
            .context(RedisSnafu {
                message: "Failed to del allow list",
            })
    }
}

/// Typed key to the allow_list
#[derive(ToRedisArgs)]
#[to_redis_args(fmt = "opentalk-signaling:room={room}:automod:allow_list")]
struct RoomAutomodAllowList {
    room: SignalingRoomId,
}
