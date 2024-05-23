// SPDX-FileCopyrightText: OpenTalk GmbH <mail@opentalk.eu>
//
// SPDX-License-Identifier: EUPL-1.2

use opentalk_signaling_core::RedisConnection;
use opentalk_types::core::{ResumptionToken, TicketToken};
use redis::{AsyncCommands as _, RedisError};
use redis_args::ToRedisArgs;
use snafu::{whatever, ResultExt as _, Snafu};

use crate::api::signaling::{resumption::ResumptionData, ticket::TicketData};

const TICKET_EXPIRY_SECONDS: u64 = 30;
const RESUMPTION_TOKEN_EXPIRY_SECONDS: u64 = 120;

/// Typed redis key for a signaling ticket containing [`TicketData`]
#[derive(Debug, Copy, Clone, ToRedisArgs)]
#[to_redis_args(fmt = "opentalk-signaling:ticket={ticket}")]
struct TicketRedisKey<'s> {
    ticket: &'s TicketToken,
}

pub(crate) async fn set_ticket_ex(
    redis_conn: &mut RedisConnection,
    ticket: &TicketToken,
    ticket_data: &TicketData,
) -> Result<(), RedisError> {
    redis_conn
        .set_ex(
            TicketRedisKey { ticket },
            ticket_data,
            TICKET_EXPIRY_SECONDS,
        )
        .await
}

pub(crate) async fn get_ticket(
    redis_conn: &mut RedisConnection,
    ticket: &TicketToken,
) -> Result<Option<TicketData>, RedisError> {
    // GETDEL available since redis 6.2.0, missing direct support by redis crate
    redis::cmd("GETDEL")
        .arg(TicketRedisKey { ticket })
        .query_async(redis_conn)
        .await
}

/// Redis key for a resumption token containing [`ResumptionData`].
#[derive(Debug, ToRedisArgs)]
#[to_redis_args(fmt = "opentalk-signaling:resumption={}")]
struct ResumptionRedisKey<'s>(&'s ResumptionToken);

pub(crate) async fn get_resumption_token_data(
    redis_conn: &mut RedisConnection,
    resumption_token: &ResumptionToken,
) -> Result<Option<ResumptionData>, RedisError> {
    redis_conn.get(ResumptionRedisKey(resumption_token)).await
}

pub(crate) async fn set_resumption_token_data_if_not_exists(
    redis_conn: &mut RedisConnection,
    resumption_token: &ResumptionToken,
    data: &ResumptionData,
) -> Result<(), RedisError> {
    redis::cmd("SET")
        .arg(ResumptionRedisKey(resumption_token))
        .arg(data)
        .arg("EX")
        .arg(RESUMPTION_TOKEN_EXPIRY_SECONDS)
        .arg("NX")
        .query_async(redis_conn)
        .await
}

#[derive(Debug, Snafu)]
pub enum ResumptionError {
    #[snafu(display("Resumption token could not be refreshed as it was used"))]
    Used,
    #[snafu(whatever)]
    Other {
        message: String,
        #[snafu(source(from(Box<dyn std::error::Error + Send + Sync>, Some)))]
        source: Option<Box<dyn std::error::Error + Send + Sync>>,
    },
}

pub(crate) async fn refresh_resumption_token(
    redis_conn: &mut RedisConnection,
    resumption_token: &ResumptionToken,
    data: &ResumptionData,
) -> Result<(), ResumptionError> {
    // Set the value with an timeout of 120 seconds (EX 120)
    // and only if it already exists
    let value: redis::Value = redis::cmd("SET")
        .arg(ResumptionRedisKey(resumption_token))
        .arg(data)
        .arg("EX")
        .arg(RESUMPTION_TOKEN_EXPIRY_SECONDS)
        .arg("XX")
        .query_async(redis_conn)
        .await
        .whatever_context("Failed to SET EX XX resumption data")?;

    match value {
        redis::Value::Nil => UsedSnafu.fail(),
        redis::Value::Okay => Ok(()),
        _ => whatever!("Unexpected redis response expected OK/nil got {:?}", value),
    }
}

pub(crate) async fn delete_resumption_token(
    redis_conn: &mut RedisConnection,
    resumption_token: &ResumptionToken,
) -> Result<bool, RedisError> {
    redis_conn.del(ResumptionRedisKey(resumption_token)).await
}
