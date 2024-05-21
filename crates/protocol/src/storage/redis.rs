// SPDX-FileCopyrightText: OpenTalk GmbH <mail@opentalk.eu>
//
// SPDX-License-Identifier: EUPL-1.2

use async_trait::async_trait;
use opentalk_signaling_core::{RedisConnection, RedisSnafu, SignalingModuleError, SignalingRoomId};
use opentalk_types::core::ParticipantId;
use redis::AsyncCommands;
use redis_args::ToRedisArgs;
use snafu::ResultExt;

use super::{protocol_storage::ProtocolStorage, InitState};
use crate::SessionInfo;

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
    async fn group_delete(&mut self, room_id: SignalingRoomId) -> Result<(), SignalingModuleError> {
        self.del(GroupKey { room_id }).await.context(RedisSnafu {
            message: "Failed to delete protocol group key",
        })
    }

    #[tracing::instrument(name = "protocol_try_start_init", skip(self))]
    async fn try_start_init(
        &mut self,
        room_id: SignalingRoomId,
    ) -> Result<Option<InitState>, SignalingModuleError> {
        let affected_entries: i64 = self
            .set_nx(InitKey { room_id }, InitState::Initializing)
            .await
            .context(RedisSnafu {
                message: "Failed to set protocol init state",
            })?;

        if affected_entries == 1 {
            Ok(None)
        } else {
            let state: InitState = self.get(InitKey { room_id }).await.context(RedisSnafu {
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
        //     .query_async::<_, Option<InitState>>(self)
        //     .await
        //     .context( RedisSnafu {message: "Failed to set protocol init state"})
    }

    #[tracing::instrument(name = "protocol_set_initialized", skip(self))]
    async fn set_initialized(&mut self, room: SignalingRoomId) -> Result<(), SignalingModuleError> {
        self.set(InitKey { room_id: room }, InitState::Initialized)
            .await
            .context(RedisSnafu {
                message: "Failed to set protocol init state to `Initialized`",
            })
    }

    #[tracing::instrument(name = "get_protocol_init_state", skip(self))]
    async fn init_get(
        &mut self,
        room_id: SignalingRoomId,
    ) -> Result<Option<InitState>, SignalingModuleError> {
        self.get(InitKey { room_id }).await.context(RedisSnafu {
            message: "Failed to get protocol init state",
        })
    }

    #[tracing::instrument(name = "delete_protocol_init_state", skip(self))]
    async fn init_delete(&mut self, room_id: SignalingRoomId) -> Result<(), SignalingModuleError> {
        self.del::<_, i64>(InitKey { room_id })
            .await
            .context(RedisSnafu {
                message: "Failed to delete protocol init key",
            })?;

        Ok(())
    }

    #[tracing::instrument(name = "get_protocol_session_info", skip(self))]
    async fn session_get(
        &mut self,
        room_id: SignalingRoomId,
        participant_id: ParticipantId,
    ) -> Result<Option<SessionInfo>, SignalingModuleError> {
        self.get(SessionInfoKey {
            room_id,
            participant_id,
        })
        .await
        .context(RedisSnafu {
            message: "Failed to get protocol session info key",
        })
    }

    #[tracing::instrument(name = "set_protocol_session_info", skip(self))]
    async fn session_set(
        &mut self,
        room_id: SignalingRoomId,
        participant_id: ParticipantId,
        session_info: &SessionInfo,
    ) -> Result<(), SignalingModuleError> {
        self.set(
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

    #[tracing::instrument(name = "delete_protocol_session_info", skip(self))]
    async fn session_delete(
        &mut self,
        room_id: SignalingRoomId,
        participant_id: ParticipantId,
    ) -> Result<Option<SessionInfo>, SignalingModuleError> {
        redis::cmd("GETDEL")
            .arg(SessionInfoKey {
                room_id,
                participant_id,
            })
            .query_async(self)
            .await
            .context(RedisSnafu {
                message: "Failed to get_del protocol session info key",
            })
    }
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

/// Contains the [`SessionInfo`] of the a participant.
#[derive(ToRedisArgs)]
#[to_redis_args(
    fmt = "opentalk-signaling:room={room_id}:participant={participant_id}:protocol-session"
)]
pub(super) struct SessionInfoKey {
    pub(super) room_id: SignalingRoomId,
    pub(super) participant_id: ParticipantId,
}
