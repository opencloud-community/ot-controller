// SPDX-FileCopyrightText: OpenTalk GmbH <mail@opentalk.eu>
//
// SPDX-License-Identifier: EUPL-1.2

use std::collections::HashMap;

use async_trait::async_trait;
use opentalk_signaling_core::{RedisConnection, RedisSnafu, SignalingModuleError, SignalingRoomId};
use opentalk_types_signaling_legal_vote::{
    tally::Tally,
    vote::{LegalVoteId, VoteOption},
};
use redis::AsyncCommands;
use redis_args::ToRedisArgs;
use snafu::ResultExt;

use crate::storage::legal_vote_storage::LegalVoteCountStorage;

/// Contains a sorted set of [`VoteOption`] each with their respective vote count.
///
/// When a vote is casted, the corresponding vote option in this list will get incremented.
/// See [`VOTE_SCRIPT`](super::VOTE_SCRIPT) for more details on the vote process.
#[derive(ToRedisArgs)]
#[to_redis_args(fmt = "opentalk-signaling:room={room_id}:vote={legal_vote_id}:vote_count")]
pub(super) struct VoteCountKey {
    pub(super) room_id: SignalingRoomId,
    pub(super) legal_vote_id: LegalVoteId,
}

#[async_trait(?Send)]
impl LegalVoteCountStorage for RedisConnection {
    #[tracing::instrument(name = "legal_vote_get_vote_count", skip(self))]
    async fn count_get(
        &mut self,
        room_id: SignalingRoomId,
        legal_vote_id: LegalVoteId,
        enable_abstain: bool,
    ) -> Result<Tally, SignalingModuleError> {
        let vote_count: HashMap<VoteOption, u64> = self
            .zrange_withscores(
                VoteCountKey {
                    room_id,
                    legal_vote_id,
                },
                0,
                -1,
            )
            .await
            .with_context(|_| RedisSnafu {
                message: format!(
                    "Failed to get the vote count for room_id:{} legal_vote_id:{}",
                    room_id, legal_vote_id
                ),
            })?;

        Ok(Tally {
            yes: *vote_count.get(&VoteOption::Yes).unwrap_or(&0),
            no: *vote_count.get(&VoteOption::No).unwrap_or(&0),
            abstain: {
                if enable_abstain {
                    Some(*vote_count.get(&VoteOption::Abstain).unwrap_or(&0))
                } else {
                    None
                }
            },
        })
    }
}
