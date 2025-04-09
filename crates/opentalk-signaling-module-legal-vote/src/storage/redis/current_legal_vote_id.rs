// SPDX-FileCopyrightText: OpenTalk GmbH <mail@opentalk.eu>
//
// SPDX-License-Identifier: EUPL-1.2

use async_trait::async_trait;
use opentalk_signaling_core::{RedisConnection, RedisSnafu, SignalingModuleError, SignalingRoomId};
use opentalk_types_signaling_legal_vote::vote::LegalVoteId;
use redis::AsyncCommands;
use redis_args::ToRedisArgs;
use snafu::ResultExt;

use crate::storage::LegalVoteCurrentStorage;

#[async_trait(?Send)]
impl LegalVoteCurrentStorage for RedisConnection {
    #[tracing::instrument(name = "legal_vote_set_current_vote_id", skip(self))]
    async fn current_vote_set(
        &mut self,
        room_id: SignalingRoomId,
        new_vote_id: LegalVoteId,
    ) -> Result<bool, SignalingModuleError> {
        // set if not exists
        let affected_entries: i64 = self
            .set_nx(CurrentVoteIdKey { room_id }, new_vote_id)
            .await
            .context(RedisSnafu {
                message: "Failed to set current vote id",
            })?;

        if affected_entries == 1 {
            Ok(true)
        } else {
            Ok(false)
        }
    }

    #[tracing::instrument(name = "legal_vote_get_current_vote_id", skip(self))]
    async fn current_vote_get(
        &mut self,
        room_id: SignalingRoomId,
    ) -> Result<Option<LegalVoteId>, SignalingModuleError> {
        self.get(CurrentVoteIdKey { room_id })
            .await
            .context(RedisSnafu {
                message: "Failed to get current vote id",
            })
    }

    #[tracing::instrument(name = "legal_vote_delete_current_vote_id", skip(self))]
    async fn current_vote_delete(
        &mut self,
        room_id: SignalingRoomId,
    ) -> Result<(), SignalingModuleError> {
        self.del(CurrentVoteIdKey { room_id })
            .await
            .context(RedisSnafu {
                message: "Failed to delete current vote id key",
            })
    }
}

/// Contains the [`VoteId`] of the active vote.
///
/// The current vote id key acts like a kind of lock. When a vote is in progress and therefore
/// this key has a value, no new vote can be started. This key gets deleted when a vote ends.
///
/// See [`END_CURRENT_VOTE_SCRIPT`](super::END_CURRENT_VOTE_SCRIPT) for more details.
#[derive(ToRedisArgs)]
#[to_redis_args(fmt = "opentalk-signaling:room={room_id}:vote:current")]
pub(super) struct CurrentVoteIdKey {
    pub(super) room_id: SignalingRoomId,
}
