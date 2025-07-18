// SPDX-FileCopyrightText: OpenTalk GmbH <mail@opentalk.eu>
//
// SPDX-License-Identifier: EUPL-1.2

use async_trait::async_trait;
use opentalk_signaling_core::{RedisConnection, RunnerId};
use opentalk_types_common::auth::{ResumptionToken, TicketToken};
use opentalk_types_signaling::ParticipantId;
use redis::AsyncCommands;
use redis_args::ToRedisArgs;
use snafu::{ResultExt as _, ensure, whatever};

use super::{
    RESUMPTION_TOKEN_EXPIRY, SignalingStorage, SignalingStorageError, TICKET_EXPIRY,
    error::{RedisSnafu, ResumptionTokenAlreadyUsedSnafu},
};
use crate::signaling::{resumption::ResumptionData, ticket::TicketData};

#[async_trait(?Send)]
impl SignalingStorage for RedisConnection {
    #[tracing::instrument(level = "debug", skip(self))]
    async fn set_ticket_ex(
        &mut self,
        ticket_token: &TicketToken,
        ticket_data: &TicketData,
    ) -> Result<(), SignalingStorageError> {
        self.set_ex(
            TicketKey(ticket_token),
            ticket_data,
            TICKET_EXPIRY.as_secs(),
        )
        .await
        .with_context(|_| RedisSnafu {
            message: "Failed to SET EX ticket data",
        })
    }

    #[tracing::instrument(level = "debug", skip(self))]
    async fn take_ticket(
        &mut self,
        ticket_token: &TicketToken,
    ) -> Result<Option<TicketData>, SignalingStorageError> {
        // GETDEL available since redis 6.2.0, missing direct support by redis crate
        redis::cmd("GETDEL")
            .arg(TicketKey(ticket_token))
            .query_async(self)
            .await
            .with_context(|_| RedisSnafu {
                message: "Failed to GETDEL ticket data",
            })
    }

    #[tracing::instrument(level = "debug", skip(self))]
    async fn get_resumption_token_data(
        &mut self,
        resumption_token: &ResumptionToken,
    ) -> Result<Option<ResumptionData>, SignalingStorageError> {
        self.get(ResumptionKey(resumption_token))
            .await
            .with_context(|_| RedisSnafu {
                message: "Failed to GET resumption token data",
            })
    }

    #[tracing::instrument(level = "debug", skip(self))]
    async fn set_resumption_token_data_if_not_exists(
        &mut self,
        resumption_token: &ResumptionToken,
        data: &ResumptionData,
    ) -> Result<(), SignalingStorageError> {
        redis::cmd("SET")
            .arg(ResumptionKey(resumption_token))
            .arg(data)
            .arg("EX")
            .arg(RESUMPTION_TOKEN_EXPIRY.as_secs())
            .arg("NX")
            .query_async(self)
            .await
            .with_context(|_| RedisSnafu {
                message: "Failed to SET EX NX resumption data",
            })
    }

    #[tracing::instrument(level = "debug", skip(self))]
    async fn refresh_resumption_token(
        &mut self,
        resumption_token: &ResumptionToken,
    ) -> Result<(), SignalingStorageError> {
        let response: i32 = self
            .expire(
                ResumptionKey(resumption_token),
                i64::try_from(RESUMPTION_TOKEN_EXPIRY.as_secs()).unwrap_or(i64::MAX),
            )
            .await
            .with_context(|_| RedisSnafu {
                message: "Failed to update expiry of resumption data",
            })?;
        const REDIS_EXPIRE_TIMER_WAS_SET_RESPONSE: i32 = 1;
        ensure!(
            response == REDIS_EXPIRE_TIMER_WAS_SET_RESPONSE,
            ResumptionTokenAlreadyUsedSnafu
        );
        Ok(())
    }

    #[tracing::instrument(level = "debug", skip(self))]
    async fn delete_resumption_token(
        &mut self,
        resumption_token: &ResumptionToken,
    ) -> Result<bool, SignalingStorageError> {
        self.del(ResumptionKey(resumption_token))
            .await
            .with_context(|_| RedisSnafu {
                message: "Failed to delete resumption token from redis",
            })
    }

    #[tracing::instrument(level = "debug", skip(self))]
    async fn try_acquire_participant_id(
        &mut self,
        participant_id: ParticipantId,
        runner_id: RunnerId,
    ) -> Result<bool, SignalingStorageError> {
        let value: redis::Value = redis::cmd("SET")
            .arg(ParticipantIdRunnerLock { id: participant_id })
            .arg(runner_id)
            .arg("NX")
            .query_async(self)
            .await
            .context(RedisSnafu {
                message: "Failed to acquire participant id",
            })?;

        match value {
            redis::Value::Nil => Ok(false),
            redis::Value::Okay => Ok(true),
            _ => whatever!(
                "Got unexpected value while acquiring runner id, value={:?}",
                value
            ),
        }
    }

    #[tracing::instrument(level = "debug", skip(self))]
    async fn participant_id_in_use(
        &mut self,
        participant_id: ParticipantId,
    ) -> Result<bool, SignalingStorageError> {
        self.exists(ParticipantIdRunnerLock { id: participant_id })
            .await
            .context(RedisSnafu {
                message: "failed to check if participant id is in use",
            })
    }

    #[tracing::instrument(level = "debug", skip(self))]
    async fn release_participant_id(
        &mut self,
        participant_id: ParticipantId,
    ) -> Result<Option<RunnerId>, SignalingStorageError> {
        redis::cmd("GETDEL")
            .arg(ParticipantIdRunnerLock { id: participant_id })
            .query_async(self)
            .await
            .context(RedisSnafu {
                message: "failed to remove participant id",
            })
    }
}

/// Typed redis key for a signaling ticket containing [`TicketData`]
#[derive(Debug, Copy, Clone, ToRedisArgs)]
#[to_redis_args(fmt = "opentalk-signaling:ticket={}")]
struct TicketKey<'s>(&'s TicketToken);

/// Redis key for a resumption token containing [`ResumptionData`].
#[derive(Debug, ToRedisArgs)]
#[to_redis_args(fmt = "opentalk-signaling:resumption={}")]
struct ResumptionKey<'s>(&'s ResumptionToken);

#[derive(Debug, ToRedisArgs)]
#[to_redis_args(fmt = "opentalk-signaling:runner:{id}")]
struct ParticipantIdRunnerLock {
    id: ParticipantId,
}

#[cfg(test)]
mod tests {
    use redis::aio::ConnectionManager;
    use serial_test::serial;

    use super::{super::test_common, *};

    async fn storage() -> RedisConnection {
        let redis_url =
            std::env::var("REDIS_ADDR").unwrap_or_else(|_| "redis://0.0.0.0:6379/".to_owned());
        let redis = redis::Client::open(redis_url).expect("Invalid redis url");

        let mut mgr = ConnectionManager::new(redis).await.unwrap();

        redis::cmd("FLUSHALL").exec_async(&mut mgr).await.unwrap();

        RedisConnection::new(mgr)
    }

    #[tokio::test]
    #[serial]
    async fn ticket_token() {
        test_common::ticket_token(&mut storage().await).await;
    }

    #[tokio::test]
    #[serial]
    async fn resumption_token() {
        test_common::resumption_token(&mut storage().await).await;
    }

    #[tokio::test]
    #[serial]
    async fn participant_runner_lock() {
        test_common::participant_runner_lock(&mut storage().await).await;
    }

    #[tokio::test]
    #[serial]
    async fn try_acquire_participant_id() {
        test_common::try_acquire_participant_id(&mut storage().await).await;
    }
}
