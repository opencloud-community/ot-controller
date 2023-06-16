// SPDX-FileCopyrightText: OpenTalk GmbH <mail@opentalk.eu>
//
// SPDX-License-Identifier: EUPL-1.2

use super::resumption::{ResumptionData, ResumptionRedisKey};
use crate::api::v1::response::ApiError;
use anyhow::Context;
use redis::AsyncCommands;
use redis_args::{FromRedisValue, ToRedisArgs};
use serde::{Deserialize, Serialize};
use signaling_core::{control, Participant, RedisConnection};
use types::core::{BreakoutRoomId, ParticipantId, ResumptionToken, RoomId, TicketToken, UserId};

/// Typed redis key for a signaling ticket containing [`TicketData`]
#[derive(Debug, Copy, Clone, ToRedisArgs)]
#[to_redis_args(fmt = "opentalk-signaling:ticket={ticket}")]
pub struct TicketRedisKey<'s> {
    pub ticket: &'s str,
}

/// Data stored behind the [`Ticket`] key.
#[derive(Debug, Clone, Deserialize, Serialize, ToRedisArgs, FromRedisValue)]
#[to_redis_args(serde)]
#[from_redis_value(serde)]
pub struct TicketData {
    pub participant_id: ParticipantId,
    pub resuming: bool,
    pub participant: Participant<UserId>,
    pub room: RoomId,
    pub breakout_room: Option<BreakoutRoomId>,
    pub resumption: ResumptionToken,
}

pub async fn start_or_continue_signaling_session(
    redis_conn: &mut RedisConnection,
    participant: Participant<UserId>,
    room: RoomId,
    breakout_room: Option<BreakoutRoomId>,
    resumption: Option<ResumptionToken>,
) -> Result<(TicketToken, ResumptionToken), ApiError> {
    let mut resuming = false;

    // Get participant id, check resumption token if it exists, if not generate random one
    let participant_id = if let Some(resumption) = resumption {
        if let Some(id) = use_resumption_token(redis_conn, participant, room, resumption).await? {
            resuming = true;
            id
        } else {
            // invalid resumption token, generate new id
            ParticipantId::generate()
        }
    } else {
        // No resumption token, generate new id
        ParticipantId::generate()
    };

    let ticket = TicketToken::generate();
    let resumption = ResumptionToken::generate();

    let ticket_data = TicketData {
        participant_id,
        resuming,
        participant,
        room,
        breakout_room,
        resumption: resumption.clone(),
    };

    // let the ticket expire in 30 seconds
    redis_conn
        .set_ex(
            TicketRedisKey {
                ticket: ticket.as_str(),
            },
            &ticket_data,
            30,
        )
        .await
        .map_err(|e| {
            log::error!("Unable to store ticket in redis, {}", e);
            ApiError::internal()
        })?;

    Ok((ticket, resumption))
}

async fn use_resumption_token(
    redis_conn: &mut RedisConnection,
    participant: Participant<UserId>,
    room: RoomId,
    token: ResumptionToken,
) -> Result<Option<ParticipantId>, ApiError> {
    let resumption_redis_key = ResumptionRedisKey(token);

    // Check for resumption data behind resumption token
    let resumption_data: Option<ResumptionData> =
        redis_conn.get(&resumption_redis_key).await.map_err(|e| {
            log::error!("Failed to fetch resumption token from redis, {}", e);
            ApiError::internal()
        })?;

    let data = if let Some(data) = resumption_data {
        data
    } else {
        return Ok(None);
    };

    if data.room != room || data.participant != participant {
        log::debug!(
            "given resumption was valid but was used in an invalid context (wrong user/room)"
        );
        return Ok(None);
    }

    if control::storage::participant_id_in_use(redis_conn, data.participant_id).await? {
        return Err(ApiError::bad_request()
            .with_code("session_running")
            .with_message("the session of the given resumption token is still running"));
    }

    if redis_conn
        .del(&resumption_redis_key)
        .await
        .context("failed to remove resumption token")?
    {
        Ok(Some(data.participant_id))
    } else {
        Err(ApiError::internal())
    }
}
