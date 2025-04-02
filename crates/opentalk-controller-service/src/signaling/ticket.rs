// SPDX-FileCopyrightText: OpenTalk GmbH <mail@opentalk.eu>
//
// SPDX-License-Identifier: EUPL-1.2

use opentalk_signaling_core::{Participant, VolatileStorage};
use opentalk_types_api_v1::error::ApiError;
use opentalk_types_common::{
    auth::{ResumptionToken, TicketToken},
    rooms::{BreakoutRoomId, RoomId},
    users::UserId,
};
use opentalk_types_signaling::ParticipantId;
use redis_args::{FromRedisValue, ToRedisArgs};
use serde::{Deserialize, Serialize};
use snafu::Report;

use crate::signaling::storage::SignalingStorageProvider;

/// Ticket data
#[derive(Debug, Clone, PartialEq, Eq, Deserialize, Serialize, ToRedisArgs, FromRedisValue)]
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
    volatile: &mut VolatileStorage,
    participant: Participant<UserId>,
    room: RoomId,
    breakout_room: Option<BreakoutRoomId>,
    resumption: Option<ResumptionToken>,
) -> Result<(TicketToken, ResumptionToken), ApiError> {
    let mut resuming = false;

    // Get participant id, check resumption token if it exists, if not generate random one
    let participant_id = if let Some(resumption) = resumption {
        if let Some(id) = use_resumption_token(volatile, participant, room, resumption).await? {
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

    let ticket = TicketToken::generate_for_room(room);
    let resumption = ResumptionToken::generate();

    let ticket_data = TicketData {
        participant_id,
        resuming,
        participant,
        room,
        breakout_room,
        resumption: resumption.clone(),
    };

    volatile
        .signaling_storage()
        .set_ticket_ex(&ticket, &ticket_data)
        .await
        .map_err(|e| {
            log::error!("Unable to store ticket in redis, {}", Report::from_error(e));
            ApiError::internal()
        })?;

    Ok((ticket, resumption))
}

async fn use_resumption_token(
    volatile: &mut VolatileStorage,
    participant: Participant<UserId>,
    room: RoomId,
    resumption_token: ResumptionToken,
) -> Result<Option<ParticipantId>, ApiError> {
    let resumption_data = volatile
        .signaling_storage()
        .get_resumption_token_data(&resumption_token)
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

    if volatile
        .signaling_storage()
        .participant_id_in_use(data.participant_id)
        .await?
    {
        return Err(ApiError::bad_request()
            .with_code("session_running")
            .with_message("the session of the given resumption token is still running"));
    }

    let delete_success = volatile
        .signaling_storage()
        .delete_resumption_token(&resumption_token)
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
