// SPDX-FileCopyrightText: OpenTalk GmbH <mail@opentalk.eu>
//
// SPDX-License-Identifier: EUPL-1.2

use async_trait::async_trait;
use opentalk_signaling_core::{RedisConnection, RedisSnafu, SignalingModuleError, SignalingRoomId};
use opentalk_types_signaling_legal_vote::{parameters::Parameters, vote::LegalVoteId};
use redis::AsyncCommands;
use redis_args::ToRedisArgs;
use snafu::ResultExt;

use crate::storage::LegalVoteParameterStorage;

#[async_trait(?Send)]
impl LegalVoteParameterStorage for RedisConnection {
    #[tracing::instrument(name = "legal_vote_set_parameters", skip(self, parameters))]
    async fn parameter_set(
        &mut self,
        room_id: SignalingRoomId,
        legal_vote_id: LegalVoteId,
        parameters: &Parameters,
    ) -> Result<(), SignalingModuleError> {
        self.set(
            VoteParametersKey {
                room_id,
                legal_vote_id,
            },
            parameters,
        )
        .await
        .with_context(|_| RedisSnafu {
            message: format!(
                "Failed to set the vote parameter for room_id:{room_id} legal_vote_id:{legal_vote_id}"
            ),
        })
    }

    #[tracing::instrument(name = "legal_vote_get_parameters", skip(self))]
    async fn parameter_get(
        &mut self,
        room_id: SignalingRoomId,
        legal_vote_id: LegalVoteId,
    ) -> Result<Option<Parameters>, SignalingModuleError> {
        self.get(VoteParametersKey {
            room_id,
            legal_vote_id,
        })
        .await
        .with_context(|_| RedisSnafu {
            message: format!(
                "Failed to get the vote parameter for room_id:{room_id} legal_vote_id:{legal_vote_id}"
            ),
        })
    }
}

/// Contains the [`Parameters`] of the a vote.
#[derive(ToRedisArgs)]
#[to_redis_args(fmt = "opentalk-signaling:room={room_id}:vote={legal_vote_id}")]
pub(super) struct VoteParametersKey {
    pub(super) room_id: SignalingRoomId,
    pub(super) legal_vote_id: LegalVoteId,
}
