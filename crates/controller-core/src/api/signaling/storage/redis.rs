// SPDX-FileCopyrightText: OpenTalk GmbH <mail@opentalk.eu>
//
// SPDX-License-Identifier: EUPL-1.2

use opentalk_signaling_core::RedisConnection;
use opentalk_types::core::TicketToken;
use redis::{AsyncCommands as _, RedisError};
use redis_args::ToRedisArgs;

use crate::api::signaling::ticket::TicketData;

const TICKET_EXPIRY_SECONDS: u64 = 30;

/// Typed redis key for a signaling ticket containing [`TicketData`]
#[derive(Debug, Copy, Clone, ToRedisArgs)]
#[to_redis_args(fmt = "opentalk-signaling:ticket={ticket}")]
pub struct TicketRedisKey<'s> {
    pub ticket: &'s TicketToken,
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
