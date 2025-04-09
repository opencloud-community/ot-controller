// SPDX-FileCopyrightText: OpenTalk GmbH <mail@opentalk.eu>
//
// SPDX-License-Identifier: EUPL-1.2

use std::collections::BTreeSet;

use async_trait::async_trait;
use opentalk_signaling_core::{RedisConnection, RedisSnafu, SignalingModuleError, SignalingRoomId};
use opentalk_types_signaling_legal_vote::vote::LegalVoteId;
use redis::AsyncCommands;
use redis_args::ToRedisArgs;
use snafu::ResultExt;

use crate::storage::LegalVoteHistoryStorage;

#[async_trait(?Send)]
impl LegalVoteHistoryStorage for RedisConnection {
    #[tracing::instrument(name = "legal_vote_get_history", skip(self))]
    async fn history_get(
        &mut self,
        room_id: SignalingRoomId,
    ) -> Result<BTreeSet<LegalVoteId>, SignalingModuleError> {
        self.smembers(VoteHistoryKey { room_id })
            .await
            .context(RedisSnafu {
                message: "Failed to get vote history",
            })
    }

    #[tracing::instrument(name = "legal_vote_history_contains", skip(self))]
    async fn history_contains(
        &mut self,
        room_id: SignalingRoomId,
        vote_id: LegalVoteId,
    ) -> Result<bool, SignalingModuleError> {
        self.sismember(VoteHistoryKey { room_id }, vote_id)
            .await
            .context(RedisSnafu {
                message: "Failed to check if vote history contains vote",
            })
    }

    #[tracing::instrument(name = "legal_vote_delete_history", skip(self))]
    async fn history_delete(
        &mut self,
        room_id: SignalingRoomId,
    ) -> Result<(), SignalingModuleError> {
        self.del(VoteHistoryKey { room_id })
            .await
            .context(RedisSnafu {
                message: "Failed to remove vote history",
            })
    }
}

/// Contains a set of [`VoteId`] from all votes that were completed since the start of this room.
///
/// When a vote is stopped or canceled, the vote id will be added to this key.
/// See [`END_CURRENT_VOTE_SCRIPT`](super::END_CURRENT_VOTE_SCRIPT) for more details.
#[derive(ToRedisArgs)]
#[to_redis_args(fmt = "opentalk-signaling:room={room_id}:vote:history")]
pub(super) struct VoteHistoryKey {
    pub(super) room_id: SignalingRoomId,
}
