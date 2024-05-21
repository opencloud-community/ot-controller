// SPDX-FileCopyrightText: OpenTalk GmbH <mail@opentalk.eu>
//
// SPDX-License-Identifier: EUPL-1.2

use std::collections::{BTreeMap, BTreeSet};

use async_trait::async_trait;
use opentalk_signaling_core::{RedisConnection, RedisSnafu, SignalingModuleError, SignalingRoomId};
use opentalk_types::signaling::polls::{state::PollsState, ChoiceId, Item, PollId};
use redis::AsyncCommands as _;
use redis_args::ToRedisArgs;
use snafu::{whatever, ResultExt as _};

use super::polls_storage::PollsStorage;

#[async_trait(?Send)]
impl PollsStorage for RedisConnection {
    #[tracing::instrument(level = "debug", skip(self))]
    async fn get_polls_state(
        &mut self,
        room: SignalingRoomId,
    ) -> Result<Option<PollsState>, SignalingModuleError> {
        self.get(PollsStateKey { room }).await.context(RedisSnafu {
            message: "Failed to get current polls state",
        })
    }

    #[tracing::instrument(level = "debug", skip(self))]
    async fn set_polls_state(
        &mut self,
        room: SignalingRoomId,
        polls_state: &PollsState,
    ) -> Result<bool, SignalingModuleError> {
        let value: redis::Value = redis::cmd("SET")
            .arg(PollsStateKey { room })
            .arg(polls_state)
            .arg("EX")
            .arg(polls_state.duration.as_secs())
            .arg("NX")
            .query_async(self)
            .await
            .context(RedisSnafu {
                message: "Failed to set current polls state",
            })?;

        match value {
            redis::Value::Okay => Ok(true),
            redis::Value::Nil => Ok(false),
            _ => whatever!("got invalid value from SET EX NX: {:?}", value),
        }
    }

    #[tracing::instrument(level = "debug", skip(self))]
    async fn delete_polls_state(
        &mut self,
        room: SignalingRoomId,
    ) -> Result<(), SignalingModuleError> {
        self.del(PollsStateKey { room }).await.context(RedisSnafu {
            message: "Failed to del current polls state",
        })
    }

    async fn delete_poll_results(
        &mut self,
        room: SignalingRoomId,
        poll_id: PollId,
    ) -> Result<(), SignalingModuleError> {
        self.del(PollResults {
            room,
            poll: poll_id,
        })
        .await
        .context(RedisSnafu {
            message: "Failed to delete results",
        })
    }

    /// Add a poll to the list
    async fn add_poll_to_list(
        &mut self,
        room: SignalingRoomId,
        poll_id: PollId,
    ) -> Result<(), SignalingModuleError> {
        self.sadd(PollList { room }, poll_id)
            .await
            .context(RedisSnafu {
                message: "Failed to sadd poll list",
            })
    }

    async fn results(
        &mut self,
        room: SignalingRoomId,
        poll: PollId,
    ) -> Result<BTreeMap<ChoiceId, u32>, SignalingModuleError> {
        self.zrange_withscores(PollResults { room, poll }, 0, -1)
            .await
            .context(RedisSnafu {
                message: "failed to zrange vote results",
            })
    }

    async fn vote(
        &mut self,
        room: SignalingRoomId,
        poll_id: PollId,
        previous_choice_ids: &BTreeSet<ChoiceId>,
        new_choice_ids: &BTreeSet<ChoiceId>,
    ) -> Result<(), SignalingModuleError> {
        // Revoke any previous vote.
        for choice_id in previous_choice_ids {
            self.zincr(
                PollResults {
                    room,
                    poll: poll_id,
                },
                u32::from(*choice_id),
                -1,
            )
            .await
            .context(RedisSnafu {
                message: "Failed to cast previous vote",
            })?;
        }

        // Apply any new vote.
        for choice_id in new_choice_ids {
            self.zincr(
                PollResults {
                    room,
                    poll: poll_id,
                },
                u32::from(*choice_id),
                1,
            )
            .await
            .context(RedisSnafu {
                message: "Failed to cast new vote",
            })?;
        }

        Ok(())
    }

    #[tracing::instrument(level = "debug", skip(self))]
    async fn poll_ids(
        &mut self,
        room: SignalingRoomId,
    ) -> Result<Vec<PollId>, SignalingModuleError> {
        self
            .smembers(PollList { room })
            .await
            .context(RedisSnafu {
                message: "Failed to get members from poll list",
            })
    }
}

/// Key to the current poll config
#[derive(ToRedisArgs)]
#[to_redis_args(fmt = "opentalk-signaling:room={room}:polls:state")]
struct PollsStateKey {
    room: SignalingRoomId,
}

/// Key to the current vote results
#[derive(ToRedisArgs)]
#[to_redis_args(fmt = "opentalk-signaling:room={room}:poll={poll}:results")]
struct PollResults {
    room: SignalingRoomId,
    poll: PollId,
}

pub(crate) async fn poll_results(
    redis_conn: &mut RedisConnection,
    room: SignalingRoomId,
    config: &PollsState,
) -> Result<Vec<Item>, SignalingModuleError> {
    let votes = redis_conn.results(room, config.id).await?;

    let votes = (0..config.choices.len())
        .map(|i| {
            let id = ChoiceId::from(i as u32);
            let count = votes.get(&id).copied().unwrap_or_default();
            Item { id, count }
        })
        .collect();

    Ok(votes)
}

/// Key to the list of all polls inside the given room
#[derive(ToRedisArgs)]
#[to_redis_args(fmt = "opentalk-signaling:room={room}:polls:list")]
struct PollList {
    room: SignalingRoomId,
}
