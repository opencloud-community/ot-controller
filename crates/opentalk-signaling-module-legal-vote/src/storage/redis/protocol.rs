// SPDX-FileCopyrightText: OpenTalk GmbH <mail@opentalk.eu>
//
// SPDX-License-Identifier: EUPL-1.2

use async_trait::async_trait;
use opentalk_signaling_core::{RedisConnection, RedisSnafu, SignalingModuleError, SignalingRoomId};
use opentalk_types_signaling_legal_vote::vote::LegalVoteId;
use redis::AsyncCommands;
use redis_args::ToRedisArgs;
use snafu::ResultExt;

use crate::storage::{legal_vote_storage::LegalVoteProtocolStorage, protocol::v1::ProtocolEntry};

#[async_trait(?Send)]
impl LegalVoteProtocolStorage for RedisConnection {
    /// Add an entry to the vote protocol of `legal_vote_id`
    #[tracing::instrument(name = "legal_vote_add_protocol_entry", skip(self, entry))]
    async fn protocol_add_entry(
        &mut self,
        room_id: SignalingRoomId,
        legal_vote_id: LegalVoteId,
        entry: ProtocolEntry,
    ) -> Result<(), SignalingModuleError> {
        self.rpush::<_, _, ()>(
            ProtocolKey {
                room_id,
                legal_vote_id,
            },
            entry,
        )
        .await
        .context(RedisSnafu {
            message: "Failed to add vote protocol entry",
        })?;

        Ok(())
    }

    /// Get the vote protocol for `legal_vote_id`
    #[tracing::instrument(name = "legal_vote_get_protocol", skip(self))]
    async fn protocol_get(
        &mut self,
        room_id: SignalingRoomId,
        legal_vote_id: LegalVoteId,
    ) -> Result<Vec<ProtocolEntry>, SignalingModuleError> {
        self.lrange(
            ProtocolKey {
                room_id,
                legal_vote_id,
            },
            0,
            -1,
        )
        .await
        .context(RedisSnafu {
            message: "Failed to get vote protocol",
        })
    }
}

/// Contains the vote protocol. The vote protocol is a list of [`ProtocolEntries`](ProtocolEntry)
/// with information about the event that happened.
#[derive(ToRedisArgs)]
#[to_redis_args(fmt = "opentalk-signaling:room={room_id}:vote={legal_vote_id}:protocol")]
pub(super) struct ProtocolKey {
    pub(super) room_id: SignalingRoomId,
    pub(super) legal_vote_id: LegalVoteId,
}
