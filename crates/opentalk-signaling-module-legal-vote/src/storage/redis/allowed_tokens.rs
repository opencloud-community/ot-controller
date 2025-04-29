// SPDX-FileCopyrightText: OpenTalk GmbH <mail@opentalk.eu>
//
// SPDX-License-Identifier: EUPL-1.2

use async_trait::async_trait;
use opentalk_signaling_core::{RedisConnection, RedisSnafu, SignalingModuleError, SignalingRoomId};
use opentalk_types_signaling_legal_vote::{token::Token, vote::LegalVoteId};
use redis::AsyncCommands;
use redis_args::ToRedisArgs;
use snafu::ResultExt;

use crate::storage::LegalVoteAllowTokenStorage;

#[async_trait(?Send)]
impl LegalVoteAllowTokenStorage for RedisConnection {
    #[tracing::instrument(name = "legal_vote_set_allowed_tokens", skip(self, allowed_tokens))]
    async fn allow_token_set(
        &mut self,
        room_id: SignalingRoomId,
        legal_vote_id: LegalVoteId,
        allowed_tokens: Vec<Token>,
    ) -> Result<(), SignalingModuleError> {
        self.sadd(
            AllowedTokensKey {
                room_id,
                legal_vote_id,
            },
            allowed_tokens,
        )
        .await
        .with_context(|_| RedisSnafu {
            message: format!(
                "Failed to set the allowed tokens for room_id:{} legal_vote_id:{}",
                room_id, legal_vote_id
            ),
        })
    }
}

/// A set of tokens that can be used to vote.
///
/// When a vote is casted, the consumed token will be removed from this set in order to proceed.
/// When a token is not contained in this set, the token is either consumed already or was never allowed to be used for voting.
///
/// See [`VOTE_SCRIPT`](super::VOTE_SCRIPT) for more details on the vote process.
#[derive(ToRedisArgs)]
#[to_redis_args(fmt = "opentalk-signaling:room={room_id}:vote={legal_vote_id}:allowed_tokens")]
pub(super) struct AllowedTokensKey {
    pub(super) room_id: SignalingRoomId,
    pub(super) legal_vote_id: LegalVoteId,
}
