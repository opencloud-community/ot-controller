// SPDX-FileCopyrightText: OpenTalk GmbH <mail@opentalk.eu>
//
// SPDX-License-Identifier: EUPL-1.2

use async_trait::async_trait;
use opentalk_signaling_core::{RedisConnection, RedisSnafu, SignalingModuleError, SignalingRoomId};
use opentalk_types::core::ParticipantId;
use redis::AsyncCommands;
use redis_args::{FromRedisValue, ToRedisArgs};
use serde::{Deserialize, Serialize};
use snafu::ResultExt;

#[async_trait(?Send)]
impl ProtocolStorage for RedisConnection {
    #[tracing::instrument(name = "set_protocol_group", skip(self))]
    async fn group_set(
        &mut self,
        room_id: SignalingRoomId,
        group_id: &str,
    ) -> Result<(), SignalingModuleError> {
        self.set(GroupKey { room_id }, group_id)
            .await
            .context(RedisSnafu {
                message: "Failed to set protocol group key",
            })
    }

    #[tracing::instrument(name = "get_protocol_group", skip(self))]
    async fn group_get(
        &mut self,
        room_id: SignalingRoomId,
    ) -> Result<Option<String>, SignalingModuleError> {
        self.get(GroupKey { room_id }).await.context(RedisSnafu {
            message: "Failed to get protocol group key",
        })
    }

    #[tracing::instrument(name = "delete_protocol_group", skip(self))]
    async fn group_delete(
        &mut self,
        room_id: SignalingRoomId,
    ) -> Result<(), SignalingModuleError> {
        self
            .del(GroupKey { room_id })
            .await
            .context(RedisSnafu {
                message: "Failed to delete protocol group key",
            })
    }
}

/// Remove all redis keys related to this room & module
#[tracing::instrument(name = "cleanup_protocol", skip(redis_conn))]
pub(crate) async fn cleanup(
    redis_conn: &mut RedisConnection,
    room_id: SignalingRoomId,
) -> Result<(), SignalingModuleError> {
    init_del(redis_conn, room_id).await?;
    redis_conn.group_delete(room_id).await?;

    Ok(())
}

/// Stores the etherpad group_id that is associated with this room.
#[derive(ToRedisArgs)]
#[to_redis_args(fmt = "opentalk-signaling:room={room_id}:protocol:group")]
pub(super) struct GroupKey {
    pub(super) room_id: SignalingRoomId,
}

/// Stores the [`InitState`] of this room.
#[derive(ToRedisArgs)]
#[to_redis_args(fmt = "opentalk-signaling:room={room_id}:protocol:init")]
struct InitKey {
    room_id: SignalingRoomId,
}

#[derive(Debug, PartialEq, Eq, Serialize, Deserialize, ToRedisArgs, FromRedisValue)]
#[to_redis_args(serde)]
#[from_redis_value(serde)]
pub enum InitState {
    Initializing,
    Initialized,
}

/// Attempts to set the room state to [`InitState::Initializing`] with a SETNX command.
///
/// If the key already holds a value, the current key gets returned without changing the state.
///
/// Behaves like a SETNX-GET redis command.
///
/// When the key was empty and the `Initializing` state was set, Ok(None) will be returned.
#[tracing::instrument(name = "protocol_try_start_init", skip(redis_conn))]
pub(crate) async fn try_start_init(
    redis_conn: &mut RedisConnection,
    room_id: SignalingRoomId,
) -> Result<Option<InitState>, SignalingModuleError> {
    let affected_entries: i64 = redis_conn
        .set_nx(InitKey { room_id }, InitState::Initializing)
        .await
        .context(RedisSnafu {
            message: "Failed to set protocol init state",
        })?;

    if affected_entries == 1 {
        Ok(None)
    } else {
        let state: InitState = redis_conn
            .get(InitKey { room_id })
            .await
            .context(RedisSnafu {
                message: "Failed to get protocol init state",
            })?;

        Ok(Some(state))
    }

    // FIXME: use this when redis 7.0 is released
    // redis::cmd("SET")
    //     .arg(InitKey { room_id })
    //     .arg(InitState::Initializing)
    //     .arg("NX")
    //     .arg("GET")
    //     .query_async::<_, Option<InitState>>(redis_conn)
    //     .await
    //     .context( RedisSnafu {message: "Failed to set protocol init state"})
}

/// Sets the room state to [`InitState::Initialized`]
#[tracing::instrument(name = "protocol_set_initialized", skip(redis_conn))]
pub(crate) async fn set_initialized(
    redis_conn: &mut RedisConnection,
    room_id: SignalingRoomId,
) -> Result<(), SignalingModuleError> {
    redis_conn
        .set(InitKey { room_id }, InitState::Initialized)
        .await
        .context(RedisSnafu {
            message: "Failed to set protocol init state to `Initialized`",
        })
}

#[tracing::instrument(name = "get_protocol_init_state", skip(redis_conn))]
pub(crate) async fn init_get(
    redis_conn: &mut RedisConnection,
    room_id: SignalingRoomId,
) -> Result<Option<InitState>, SignalingModuleError> {
    redis_conn
        .get(InitKey { room_id })
        .await
        .context(RedisSnafu {
            message: "Failed to get protocol init state",
        })
}

#[tracing::instrument(name = "delete_protocol_init_state", skip(redis_conn))]
pub(crate) async fn init_del(
    redis_conn: &mut RedisConnection,
    room_id: SignalingRoomId,
) -> Result<(), SignalingModuleError> {
    redis_conn
        .del::<_, i64>(InitKey { room_id })
        .await
        .context(RedisSnafu {
            message: "Failed to delete protocol init key",
        })?;

    Ok(())
}

use super::protocol_storage::ProtocolStorage;
use crate::SessionInfo;

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
pub(crate) async fn session_set(
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
pub(crate) async fn session_get(
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
pub(crate) async fn session_get_del(
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
