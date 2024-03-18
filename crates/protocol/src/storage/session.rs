// SPDX-FileCopyrightText: OpenTalk GmbH <mail@opentalk.eu>
//
// SPDX-License-Identifier: EUPL-1.2

use crate::SessionInfo;
use opentalk_signaling_core::{RedisConnection, RedisSnafu, SignalingModuleError, SignalingRoomId};
use opentalk_types::core::ParticipantId;
use redis::AsyncCommands;
use redis_args::ToRedisArgs;
use snafu::ResultExt;

/// Contains the [`SessionInfo`] of the a participant.
#[derive(ToRedisArgs)]
#[to_redis_args(
    fmt = "opentalk-signaling:room={room_id}:participant={participant_id}:protocol-session"
)]
pub(super) struct SessionInfoKey {
    pub(super) room_id: SignalingRoomId,
    pub(super) participant_id: ParticipantId,
}

#[tracing::instrument(name = "set_protocol_session_info", skip(redis_conn))]
pub(crate) async fn set(
    redis_conn: &mut RedisConnection,
    room_id: SignalingRoomId,
    participant_id: ParticipantId,
    session_info: &SessionInfo,
) -> Result<(), SignalingModuleError> {
    redis_conn
        .set(
            SessionInfoKey {
                room_id,
                participant_id,
            },
            session_info,
        )
        .await
        .context(RedisSnafu {
            message: "Failed to set protocol session info key",
        })
}

#[tracing::instrument(name = "get_protocol_session_info", skip(redis_conn))]
pub(crate) async fn get(
    redis_conn: &mut RedisConnection,
    room_id: SignalingRoomId,
    participant_id: ParticipantId,
) -> Result<Option<SessionInfo>, SignalingModuleError> {
    redis_conn
        .get(SessionInfoKey {
            room_id,
            participant_id,
        })
        .await
        .context(RedisSnafu {
            message: "Failed to get protocol session info key",
        })
}

#[tracing::instrument(name = "get_del_protocol_session_info", skip(redis_conn))]
pub(crate) async fn get_del(
    redis_conn: &mut RedisConnection,
    room_id: SignalingRoomId,
    participant_id: ParticipantId,
) -> Result<Option<SessionInfo>, SignalingModuleError> {
    redis::cmd("GETDEL")
        .arg(SessionInfoKey {
            room_id,
            participant_id,
        })
        .query_async(redis_conn)
        .await
        .context(RedisSnafu {
            message: "Failed to get_del protocol session info key",
        })
}
