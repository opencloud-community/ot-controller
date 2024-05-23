// SPDX-FileCopyrightText: OpenTalk GmbH <mail@opentalk.eu>
//
// SPDX-License-Identifier: EUPL-1.2

use opentalk_signaling_core::{control, Participant, RedisConnection};
use opentalk_types::{
    api::error::ApiError,
    core::{BreakoutRoomId, ParticipantId, ResumptionToken, RoomId, TicketToken, UserId},
};
use redis_args::{FromRedisValue, ToRedisArgs};
use serde::{Deserialize, Serialize};
use snafu::Report;

use super::storage::{delete_resumption_token, get_resumption_token_data, set_ticket_ex};

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

    set_ticket_ex(redis_conn, &ticket, &ticket_data)
        .await
        .map_err(|e| {
            log::error!("Unable to store ticket in redis, {}", Report::from_error(e));
            ApiError::internal()
        })?;

    Ok((ticket, resumption))
}

async fn use_resumption_token(
    redis_conn: &mut RedisConnection,
    participant: Participant<UserId>,
    room: RoomId,
    resumption_token: ResumptionToken,
) -> Result<Option<ParticipantId>, ApiError> {
    let resumption_data = get_resumption_token_data(redis_conn, &resumption_token)
        .await
        .map_err(|e| {
            log::error!(
                "Failed to fetch resumption token from storage, {}",
                Report::from_error(e)
            );
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

    let delete_success = delete_resumption_token(redis_conn, &resumption_token)
        .await
        .map_err(|e| {
            log::warn!("Internal error: {}", Report::from_error(e));
            ApiError::internal()
        })?;
    if delete_success {
        Ok(Some(data.participant_id))
    } else {
        Err(ApiError::internal())
    }
}
